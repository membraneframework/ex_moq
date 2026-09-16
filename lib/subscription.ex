defmodule ExMoQ.Subscription do
  @type t :: %__MODULE__{
          priority: 0..255 | nil,
          group_start: non_neg_integer() | nil,
          latency_ns: non_neg_integer() | nil
        }
  defstruct [:priority, :group_start, :latency_ns]
end
