defmodule ExMoQ.Subscription do
  @moduledoc """
  Parameters of a MoQ subscription. 
  For a description of the fields, see https://docs.rs/moq-net/latest/moq_net/track/struct.Subscription.html
  """

  @type t :: %__MODULE__{
          priority: 0..255 | nil,
          group_start: non_neg_integer() | nil,
          latency_ns: non_neg_integer() | nil
        }
  defstruct [:priority, :group_start, :latency_ns]
end
