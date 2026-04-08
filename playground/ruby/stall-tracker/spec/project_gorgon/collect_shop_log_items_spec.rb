require 'project_gorgon'

RSpec.describe ProjectGorgon::CollectShopLogItems do
  subject(:collector) { described_class.new(paths) }

  let(:paths) do
    [
      'spec/fixtures/Player.log',
      'spec/fixtures/Player-prev.log'
    ]
  end

  describe "#items" do
    subject { collector.items }

    it { is_expected.to be_a(Hash) }
  end

  describe "#player_logs" do
    subject { collector.player_logs }

    it { is_expected.to be_a(Array) }
    its(:size) { is_expected.to eq(2) }
  end
end
