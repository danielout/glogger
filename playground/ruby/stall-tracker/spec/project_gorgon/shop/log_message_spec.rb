require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::LogMessage do
  describe ".for" do
    subject { described_class.for(item: line, index: 0) }

    context "when given unknown log line" do
      let(:line) do
        "Sat Mar 28 15:39 - This log line is not yet know"
      end

      it { is_expected.to be_a(ProjectGorgon::Shop::LogMessage::Unknown) }
    end

    context "when given known log line" do
      let(:line) do
        "Sat Mar 28 15:39 - Deradon removed Decent Horseshoes from shop"
      end

      it { is_expected.to be_a(ProjectGorgon::Shop::LogMessage::Removed) }
    end
  end
end
