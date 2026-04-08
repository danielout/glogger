require 'date'

class ProjectGorgon::Shop::LogMessage::Base
  # @note To be implemented by subclasses
  PATTERN = /
    \A
    (?<example>THIS.SAMPLE.PATTERN.MUST.NOT.MATCH)
    \z
  /x

  LINE_PATTERN = /
    (?<timestamp>
      [A-Z][a-z]{1,3}\s+  # Tue
      [A-Z][a-z]{1,3}\s+  # Apr
      \d+\s+              # 7
      \d{2}:\d{2}         # 16:25
    )
    \s+-\s+
    (?<message>.*)
  /x

  attr_reader :body, :index

  # @param body [String]
  # @param index [Integer] represents the position of the log message in the shop log.
  #                        Used to differentiate unique messages.
  def initialize(body:, index:)
    @body = body
    @index = index
  end

  class << self
    # @return [nil, ProjectGorgon::Shop::LogMessage::Base]
    #   nil if given string does not match the message type;
    #   a {ProjectGorgon::Shop::LogMessage::<Type>} if it matches.
    def match(string, index)
      message = new(body: string, index:)

      message.match.any? ? message : nil
    end
  end

  # @return [DateTime]
  def date
    @date ||= DateTime.parse(outer[:timestamp])
  end

  # @return [String]
  def message
    return "" unless outer

    outer[:message] || ""
  end

  # @return [Symbol]
  def action = (match["action"] || 'unknown').to_sym

  # @return [String]
  def item = match["item"]

  # @return [String]
  def player = match["player"]

  # @return [Integer]
  def price_total = match["price_total"]&.to_i

  # @return [Integer]
  def price_unit = match["price_unit"]&.to_i

  # @return [Integer]
  def quantity = match["quantity"]&.to_i

  # @return [String]
  def rest = match["rest"].to_s

  # @return [Hash]
  # def match
  #   ALL_PATTERNS.each do |pattern|
  #     match = message.match(pattern)
  #
  #     return match.named_captures if match
  #   end
  #
  #   { "action" => "unknown" }
  # end

  # @return [Boolean]
  def bought_action? = false

  # @return [Boolean]
  def known_action? = false

  # @return [Boolean] true if the action is taken by the shop owner; false otherwise
  def owner_action? = false

  # @return [Hash]
  def match
    match_data = message.match(self.class::PATTERN)

    if match_data
      match_data.named_captures
    else
      {}
    end
  end

  def ==(other)
    other.is_a?(self.class) && body == other.body && index == other.index
  end

  alias eql? ==

  def hash
    "#{body} - #{index}".hash
  end

  def to_csv_row
    [
      date,
      player,
      action,
      item,
      price_unit,
      quantity,
      price_total
    ]
  end

  # private

  def outer
    @outer ||= body.match(LINE_PATTERN)
  end
end
