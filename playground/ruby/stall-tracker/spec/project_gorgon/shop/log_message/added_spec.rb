require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::LogMessage::Added do
  subject(:message) { described_class.new(body: line, index: 0) }

  let(:line) do
    "Sat Mar 28 15:39 - Deradon added Quality Mystic Saddlebag to shop"
  end

  its(:bought_action?) { is_expected.to be(false) }
  its(:known_action?) { is_expected.to be(true) }
  its(:owner_action?) { is_expected.to be(true) }

  its(:action) { is_expected.to eq(:added) }
  its(:item) { is_expected.to eq("Quality Mystic Saddlebag") }
  its(:player) { is_expected.to eq("Deradon") }
  its(:price_total) { is_expected.to be_nil }
  its(:price_unit) { is_expected.to be_nil }
  its(:quantity) { is_expected.to eq(1) }
end
