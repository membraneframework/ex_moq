defmodule ExMoQ.Test.Relay do
  @moduledoc """
  Runs a MoQ relay for ExUnit integration tests.

  `start_supervised!/1` starts a relay under the current test supervisor and blocks
  until it accepts connections:

      relay = ExMoQ.Test.Relay.start_supervised!()
      {:ok, session} = ExMoQ.Native.create_session(relay.url, self(), relay.disable_tls_verify?)

  Call it in `setup_all` for a relay shared by the test module, or inside a
  single test when it needs its own instance.

  The relay runs as a `MuonTrap.Daemon`, which guarantees the OS process is
  torn down with the supervisor. Muontrap is an optional dependency of
  `:ex_moq`, so the host project must depend on it directly:
  `{:muontrap, "~> 1.8", only: :test}`.

  [moq-relay](https://doc.moq.dev/bin/relay) must be available in the environment:
  For how the binary path gets resolved, see `find_binary/1`.

  The relay listens on TCP for lossless transport, but groups can still be
  dropped as part of eviction policies.
  """

  @ready_timeout_ms 15_000
  @probe_interval_ms 100

  @type relay :: %{url: String.t(), disable_tls_verify?: boolean()}
  @type option :: {:binary, Path.t()}

  @doc """
  Starts a relay under the ExUnit test supervisor and blocks until it
  accepts connections.

  The relay can be stopped mid-test with `stop_supervised!(#{inspect(__MODULE__)})`,
  e.g. to observe session-drop handling.

  Raises if no moq-relay binary is found, or if the relay exits or does not
  accept connections within #{@ready_timeout_ms} ms.
  """
  @spec start_supervised!([option()]) :: relay()
  def start_supervised!(opts \\ []) do
    binary =
      find_binary(opts) ||
        raise """
        no moq-relay binary for the integration tests; provide one of:
          * the :binary option — path to a moq-relay binary
          * MOQ_RELAY — path to a moq-relay binary
          * moq-relay on $PATH (e.g. installed with `cargo install moq-relay`)
        """

    port_number = free_port!()
    pid = ExUnit.Callbacks.start_supervised!(daemon_spec(binary, port_number))
    await_ready!(port_number, pid)
    %{url: "tcp://127.0.0.1:#{port_number}", disable_tls_verify?: false}
  end

  @doc """
  Resolves the path of the moq-relay binary from the `:binary` option, the
  `$MOQ_RELAY` environment variable, or `$PATH`; returns `nil` if none of them
  yields an executable.
  """
  @spec find_binary([option()]) :: Path.t() | nil
  def find_binary(opts \\ []) do
    binary = opts[:binary] || System.get_env("MOQ_RELAY") || "moq-relay"
    System.find_executable(binary)
  end

  defp daemon_spec(binary, port_number) do
    args = [
      "--log-level",
      "info",
      "--server-tcp-bind",
      "127.0.0.1:#{port_number}",
      "--auth-public",
      ""
    ]

    opts = [
      stderr_to_stdout: true,
      log_output: :debug,
      log_prefix: "moq-relay: ",
      exit_status_to_reason: &{:moq_relay_exited, &1}
    ]

    Supervisor.child_spec({MuonTrap.Daemon, [binary, args, opts]},
      id: {__MODULE__, port_number},
      restart: :temporary
    )
  end

  defp free_port!() do
    {:ok, socket} = :gen_tcp.listen(0, reuseaddr: true)
    {:ok, port_number} = :inet.port(socket)
    :ok = :gen_tcp.close(socket)
    port_number
  end

  defp await_ready!(port_number, pid) do
    await_ready_loop(port_number, pid, System.monotonic_time(:millisecond) + @ready_timeout_ms)
  end

  defp await_ready_loop(port_number, pid, deadline) do
    cond do
      not Process.alive?(pid) ->
        raise "moq-relay exited before accepting connections"

      probe(port_number) ->
        :ok

      System.monotonic_time(:millisecond) > deadline ->
        raise "moq-relay did not become ready within #{@ready_timeout_ms} ms"

      true ->
        Process.sleep(@probe_interval_ms)
        await_ready_loop(port_number, pid, deadline)
    end
  end

  defp probe(port_number) do
    case :gen_tcp.connect(~c"127.0.0.1", port_number, [:binary, active: false], 1_000) do
      {:ok, socket} ->
        :gen_tcp.close(socket)
        true

      {:error, _reason} ->
        false
    end
  end
end
