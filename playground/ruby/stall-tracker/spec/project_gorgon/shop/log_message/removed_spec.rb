require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::LogMessage::Removed do
  subject(:message) { described_class.new(body: line, index: 0) }

  let(:line) do
    "Sat Mar 28 15:39 - Deradon removed Decent Horseshoes from shop"
  end

  its(:bought_action?) { is_expected.to be(false) }
  its(:known_action?) { is_expected.to be(true) }
  its(:owner_action?) { is_expected.to be(true) }

  its(:action) { is_expected.to eq(:removed) }
  its(:item) { is_expected.to eq("Decent Horseshoes") }
  its(:player) { is_expected.to eq("Deradon") }
  its(:price_total) { is_expected.to be_nil }
  its(:price_unit) { is_expected.to be_nil }
  its(:quantity) { is_expected.to eq(1) }

  context "when removing multiple" do
    let(:line) do
      "Sat Mar 28 15:39 - Deradon removed Barley Seeds x36 from shop"
    end

    its(:item) { is_expected.to eq("Barley Seeds") }
    its(:quantity) { is_expected.to eq(36) }
  end
end
