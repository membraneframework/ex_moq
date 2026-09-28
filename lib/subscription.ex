defmodule ExMoQ.Subscription do
  @moduledoc """
  Parameters of a track subscription.

  Fields map onto [`moq_net::track::Subscription`](https://docs.rs/moq-net/0.2.17/moq_net/track/struct.Subscription.html).

    * `priority` - delivery priority, higher is sent first.
      `nil` uses the [hang default](https://docs.rs/hang/0.20.9/hang/catalog/constant.PRIORITY.html)
      for the track's media kind.
    * `group_start` - first group to deliver. `nil` starts at the first group the
      publisher serves: the latest one, or an older one if another subscription
      to the same track in this session requested an earlier `group_start`.
    * `latency_ns` - maps to `latency_max`: how old a non-latest group may get
      before it is skipped. `0` skips immediately, so a catch-up join needs a
      budget covering the backlog. Defaults to `0`.
  """

  @type t :: %__MODULE__{
          priority: 0..255 | nil,
          group_start: non_neg_integer() | nil,
          latency_ns: non_neg_integer()
        }
  defstruct priority: nil, group_start: nil, latency_ns: 0
end
