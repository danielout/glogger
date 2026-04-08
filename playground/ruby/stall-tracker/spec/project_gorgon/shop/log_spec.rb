require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::Log do
  subject(:shop_log) { described_class.new(line) }

  let(:line) do
    File.read('spec/fixtures/shop_log_line.txt')
  end

  its(:time) { is_expected.to eq("19:40:40") }
  its(:source) { is_expected.to eq("LocalPlayer") }
  its(:method) { is_expected.to eq("ProcessBook") }
  its(:raw_args) { is_expected.to be_a(String) }
  its(:args) { is_expected.to be_a(MatchData) }

  describe "its args" do
    its(:title) { is_expected.to eq("Today's Shop Logs") }
    its(:log_messages) { is_expected.to be_a(Array) }
  end

  describe "#owner" do
    subject { shop_log.owner }

    it { is_expected.to eq("Deradon") }

    context "when no owner action if available" do
      let(:line) do
        File.read('spec/fixtures/shop_log_line-without_owner.txt')
      end

      it { is_expected.to be_nil }
    end
  end

  describe "#log_messages" do
    subject(:log_messages) { shop_log.log_messages }

    it "is sorted by oldest to newest" do
      expect(log_messages.first.date < log_messages.last.date).to be(true)
    end
  end
end
