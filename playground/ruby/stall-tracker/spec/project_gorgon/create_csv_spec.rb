require 'project_gorgon'

RSpec.describe ProjectGorgon::CreateCsv do
  subject(:create_csv) { described_class.new(messages, name) }

  let(:name) { "shop_log_export" }
  let(:csv) { instance_double(CSV) }

  let(:item) do
    instance_double(
      ProjectGorgon::Shop::LogMessage::Base,
      to_csv_row: ["2026-03-28 15:09", "AlestiarWolf", "bought", "Nice Saddle", 4000, 1, 4000]
    )
  end

  let(:other_item) do
    instance_double(
      ProjectGorgon::Shop::LogMessage::Base,
      to_csv_row: ["2026-03-28 15:10", "Deradon", "collected", nil, nil, nil, 14_500]
    )
  end

  let(:messages) { [item, other_item] }

  describe "#initialize" do
    its(:messages) { is_expected.to eq(messages) }
    its(:name) { is_expected.to eq(name) }
  end

  describe "#write_csv" do
    it "opens the csv with headers and writes each row" do
      allow(csv).to receive(:<<)
      allow(CSV).to receive(:open).with(
        "shop_log_export.csv",
        "w",
        write_headers: true,
        headers: described_class::CSV_HEADERS
      ).and_yield(csv)

      create_csv.write_csv

      expect(csv).to have_received(:<<).with(item.to_csv_row).ordered
      expect(csv).to have_received(:<<).with(other_item.to_csv_row).ordered
    end
  end
end
