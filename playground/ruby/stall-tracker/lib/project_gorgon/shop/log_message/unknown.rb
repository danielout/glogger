class ProjectGorgon::Shop::LogMessage::Unknown < ProjectGorgon::Shop::LogMessage::Base
  PATTERN = /
    \A
    (?<rest>.*)
    \z
  /x

  def known_action? = false
  def action = :unknown
end
