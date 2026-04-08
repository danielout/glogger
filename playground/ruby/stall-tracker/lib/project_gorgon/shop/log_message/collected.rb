class ProjectGorgon::Shop::LogMessage::Collected < ProjectGorgon::Shop::LogMessage::Base
  PATTERN = /
    \A
    (?<player>\S+)
    \s+(?<action>collected)\s+
    (?<price_total>\d+)
    \s+Councils\s+from\s+customer\s+purchases
    \z
  /x

  def known_action? = true
  def owner_action? = true
end
