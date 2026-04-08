class ProjectGorgon::Shop::LogMessage::Bought < ProjectGorgon::Shop::LogMessage::Base
  PATTERN = /
    \A
    (?<player>\S+)
    \s+(?<action>bought)\s+
    (?<item>.+?)
    \s?x?(?<quantity>\d+)?
    \s+at\s+a\s+cost\s+of\s+
    (?<price_unit>\d+)
    \s+per\s+(?<quantity_unit>\d+)
    \s+=\s+
    (?<price_total>\d+)
    \z
  /x

  def bought_action? = true
  def known_action? = true

  def price_unit
    super.to_f / quantity_unit
  end

  def quantity
    super || 1
  end

  def quantity_unit
    match["quantity_unit"].to_i
  end
end
