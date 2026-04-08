require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::LogMessage::Configured do
  subject(:message) { described_class.new(body: line, index: 0) }

  let(:line) do
    "Sat Mar 28 13:30 - Deradon configured Decent Horseshoes to cost 3500 per 1"
  end

  its(:bought_action?) { is_expected.to be(false) }
  its(:known_action?) { is_expected.to be(true) }
  its(:owner_action?) { is_expected.to be(true) }

  its(:action) { is_expected.to eq(:configured) }
  its(:item) { is_expected.to eq("Decent Horseshoes") }
  its(:player) { is_expected.to eq("Deradon") }
  its(:price_total) { is_expected.to eq(3500) }
  its(:price_unit) { is_expected.to eq(3500) }
  its(:rest) { is_expected.to eq("") }
  its(:quantity) { is_expected.to eq(1) }

  context "when quantity to be sold is > 1" do
    let(:line) do
      "Sat Mar 28 13:30 - Deradon configured Barley Seedsx36 to cost 3000 per 2. Item can only be purchased by Wogan."
    end

    its(:item) { is_expected.to eq("Barley Seeds") }
    its(:player) { is_expected.to eq("Deradon") }
    its(:price_total) { is_expected.to eq(1500 * 36) }
    its(:price_unit) { is_expected.to eq(1500) }
    its(:rest) { is_expected.to eq("Item can only be purchased by Wogan.") }
    its(:quantity) { is_expected.to eq(36) }
  end
end
