require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::LogMessage::Unknown do
  subject(:message) { described_class.new(body: line, index: 0) }

  let(:line) do
    "Sat Mar 28 15:39 - This is an unknown log message"
  end

  its(:bought_action?) { is_expected.to be(false) }
  its(:known_action?) { is_expected.to be(false) }
  its(:owner_action?) { is_expected.to be(false) }

  its(:action) { is_expected.to eq(:unknown) }
  its(:item) { is_expected.to be_nil }
  its(:player) { is_expected.to be_nil }
  its(:price_total) { is_expected.to be_nil }
  its(:price_unit) { is_expected.to be_nil }
  its(:rest) { is_expected.to eq("This is an unknown log message") }
  its(:quantity) { is_expected.to be_nil }
end
