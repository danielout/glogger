class ProjectGorgon::Shop::LogMessage::Configured < ProjectGorgon::Shop::LogMessage::Base
  PATTERN = /
    \A
    (?<player>\S+)
    \s+(?<action>configured)\s+
    (?<item>.+?)
    x?(?<quantity>\d+)?
    \s+to\s+cost\s+
    (?<price_unit>\d+)
    \s+per\s+(?<quantity_unit>\d+)\.?\s*
    (?<rest>.*)
    \z
  /x

  def known_action? = true
  def owner_action? = true

  def price_unit
    super.to_f / quantity_unit
  end

  def price_total
    quantity * price_unit
  end

  def quantity
    super || 1
  end

  def quantity_unit
    match["quantity_unit"].to_i
  end
end
