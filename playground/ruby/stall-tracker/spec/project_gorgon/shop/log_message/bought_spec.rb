require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::LogMessage::Bought do
  subject(:message) { described_class.new(body: line, index: 0) }

  let(:line) do
    "Mon Apr 6 02:16 - Kork bought Orcish Spell Pouch at a cost of 450 per 1 = 450"
  end

  its(:bought_action?) { is_expected.to be(true) }
  its(:known_action?) { is_expected.to be(true) }
  its(:owner_action?) { is_expected.to be(false) }

  its(:action) { is_expected.to eq(:bought) }
  its(:item) { is_expected.to eq("Orcish Spell Pouch") }
  its(:player) { is_expected.to eq("Kork") }
  its(:price_total) { is_expected.to eq(450) }
  its(:price_unit) { is_expected.to eq(450) }
  its(:quantity) { is_expected.to eq(1) }
  its(:quantity_unit) { is_expected.to eq(1) }

  context "when buying multiple" do
    let(:line) do
      "Mon Apr 6 15:38 - Zangariel bought Orcish Spell Pouch x12 at a cost of 450 per 1 = 5400"
    end

    its(:item) { is_expected.to eq("Orcish Spell Pouch") }
    its(:price_total) { is_expected.to eq(5400) }
    its(:price_unit) { is_expected.to eq(450) }
    its(:quantity) { is_expected.to eq(12) }
    its(:quantity_unit) { is_expected.to eq(1) }
  end

  context "when quantity sold is > 1" do
    let(:line) do
      "Tue Apr 7 20:10 - Wogan bought Barley Seeds x10 at a cost of 3 per 2 = 15"
    end

    its(:item) { is_expected.to eq("Barley Seeds") }
    its(:price_total) { is_expected.to eq(15) }
    its(:price_unit) { is_expected.to eq(1.5) }
    its(:quantity) { is_expected.to eq(10) }
    its(:quantity_unit) { is_expected.to eq(2) }
  end
end
