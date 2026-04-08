class ProjectGorgon::Shop::LogMessage::Added < ProjectGorgon::Shop::LogMessage::Base
  PATTERN = /
    \A
    (?<player>\S+)
    \s+(?<action>added)\s+
    (?<item>.+)
    \s+to\s+shop
    \z
  /x

  def known_action? = true
  def owner_action? = true

  # @todo Check logs for syntax
  def quantity = 1
end
