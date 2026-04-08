require 'project_gorgon'

RSpec.describe ProjectGorgon::Shop::LogMessage::Base do
  subject(:message) { described_class.new(body: line, index: 0) }

  let(:line) do
    "Sat Mar 28 15:39 - Deradon removed Decent Horseshoes from shop"
  end

  describe ".match" do
    context "when given matching log line" do
      subject do
        described_class.match(
          "Sat Mar 28 15:39 - THIS.SAMPLE.PATTERN.MUST.NOT.MATCH",
          0
        )
      end

      it { is_expected.to be_a(described_class) }
    end

    context "when given non-matching log line" do
      subject { described_class.match(line, 0) }

      it { is_expected.to be_nil }
    end
  end

  its(:body) { is_expected.to eq(line) }
  its(:date) { is_expected.to eq(DateTime.new(Date.today.year, 3, 28, 15, 39)) }
  its(:message) { is_expected.to eq("Deradon removed Decent Horseshoes from shop") }

  its(:bought_action?) { is_expected.to be(false) }
  its(:known_action?) { is_expected.to be(false) }
  its(:owner_action?) { is_expected.to be(false) }

  its(:action) { is_expected.to eq(:unknown) }
  its(:item) { is_expected.to be_nil }
  its(:player) { is_expected.to be_nil }
  its(:price_total) { is_expected.to be_nil }
  its(:price_unit) { is_expected.to be_nil }
  its(:quantity) { is_expected.to be_nil }
  its(:rest) { is_expected.to eq("") }

  describe "#==" do
    subject { message == other_message }

    let(:message) { described_class.new(body:, index: 0) }
    let(:other_message) { described_class.new(body: other_body, index: 0) }

    context "when body is different" do
      let(:body) { "Lorem" }
      let(:other_body) { "Ipsum" }

      it { is_expected.to be(false) }
    end

    context "when body is the same" do
      let(:body) { "Lorem" }
      let(:other_body) { "Lorem" }

      it { is_expected.to be(true) }
    end

    context "when the index is different" do
      let(:other_message) { described_class.new(body: other_body, index: 1) }

      let(:body) { "Lorem" }
      let(:other_body) { "Lorem" }

      it { is_expected.to be(false) }
    end
  end

  describe "#hash" do
    let(:message) { described_class.new(body:, index: 0) }
    let(:other_message) { described_class.new(body: other_body, index: 0) }

    context "when body is different" do
      let(:body) { "Lorem" }
      let(:other_body) { "Ipsum" }

      it "is different" do
        expect(message.hash).not_to eq(other_message.hash)
      end
    end

    context "when body is the same" do
      let(:body) { "Lorem" }
      let(:other_body) { "Lorem" }

      it "is the same" do
        expect(message.hash).to eq(other_message.hash)
      end
    end

    context "when the index is different" do
      let(:other_message) { described_class.new(body: other_body, index: 1) }

      let(:body) { "Lorem" }
      let(:other_body) { "Lorem" }

      it "is different" do
        expect(message.hash).not_to eq(other_message.hash)
      end
    end
  end
end
