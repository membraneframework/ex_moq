defmodule ExMoQ.Subscription do
  @moduledoc """
  Parameters of a track subscription.

  Fields map onto [`moq_net::track::Subscription`](https://docs.rs/moq-net/latest/moq_net/track/struct.Subscription.html).

    * `priority` - delivery priority, higher is sent first.
      `nil` uses the [hang default](https://docs.rs/hang/latest/hang/catalog/constant.PRIORITY.html)
      for the track's media kind.
    * `group_start` - first group to deliver. `nil` joins at the latest group.
    * `latency_ns` - maps to `latency_max`: how old a non-latest group may get
      before it is skipped. `0` skips immediately, so a catch-up join needs a
      budget covering the backlog. `nil` uses the consumer's `latency_ns`.
  """

  @type t :: %__MODULE__{
          priority: 0..255 | nil,
          group_start: non_neg_integer() | nil,
          latency_ns: non_neg_integer() | nil
        }
  defstruct [:priority, :group_start, :latency_ns]
end
