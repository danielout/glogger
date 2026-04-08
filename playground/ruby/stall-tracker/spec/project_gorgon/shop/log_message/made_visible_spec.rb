require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::LogMessage::MadeVisible do
  subject(:message) { described_class.new(body: line, index: 0) }

  let(:line) do
    "Sat Mar 28 13:04 - Deradon made Amazing Reins visible in shop at a cost of 6000 per 1"
  end

  its(:bought_action?) { is_expected.to be(false) }
  its(:known_action?) { is_expected.to be(true) }
  its(:owner_action?) { is_expected.to be(true) }

  its(:action) { is_expected.to eq(:visible) }
  its(:item) { is_expected.to eq("Amazing Reins") }
  its(:player) { is_expected.to eq("Deradon") }
  its(:price_total) { is_expected.to eq(6000) }
  its(:price_unit) { is_expected.to eq(6000) }
  its(:rest) { is_expected.to eq("") }
  its(:quantity) { is_expected.to eq(1) }

  context "when quantity to be sold is > 1" do
    let(:line) do
      # rubocop:disable Layout/LineLength
      "Sat Mar 28 13:30 - Deradon made Barley Seedsx36 visible in shop at a cost of 3000 per 2. Item can only be purchased by Wogan."
      # rubocop:enable Layout/LineLength
    end

    its(:item) { is_expected.to eq("Barley Seeds") }
    its(:player) { is_expected.to eq("Deradon") }
    its(:price_total) { is_expected.to eq(1500 * 36) }
    its(:price_unit) { is_expected.to eq(1500) }
    its(:rest) { is_expected.to eq("Item can only be purchased by Wogan.") }
    its(:quantity) { is_expected.to eq(36) }
  end
end
