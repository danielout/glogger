require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::LogMessage::Collected do
  subject(:message) { described_class.new(body: line, index: 0) }

  let(:line) do
    "Sat Mar 28 14:13 - Deradon collected 30500 Councils from customer purchases"
  end

  its(:bought_action?) { is_expected.to be(false) }
  its(:known_action?) { is_expected.to be(true) }
  its(:owner_action?) { is_expected.to be(true) }

  its(:action) { is_expected.to eq(:collected) }
  its(:item) { is_expected.to be_nil }
  its(:player) { is_expected.to eq("Deradon") }
  its(:price_total) { is_expected.to eq(30_500) }
  its(:price_unit) { is_expected.to be_nil }
  its(:quantity) { is_expected.to be_nil }
end
