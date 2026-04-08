class ProjectGorgon::Shop::LogMessage::Removed < ProjectGorgon::Shop::LogMessage::Base
  PATTERN = /
    \A
    (?<player>\S+)
    \s+(?<action>removed)\s+
    (?<item>.+?)
    \s?x?(?<quantity>\d+)?
    \s+from\s+shop
    \z
  /x

  def known_action? = true
  def owner_action? = true

  def quantity
    super || 1
  end
end
