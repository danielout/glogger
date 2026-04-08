require 'project_gorgon'

RSpec.describe ProjectGorgon::PlayerLog do
  subject(:player_log) { described_class.new(path) }

  let(:path) { 'spec/fixtures/Player.log' }

  describe "#today_shop_logs" do
    subject(:today_shop_logs) { player_log.today_shop_logs }

    it { is_expected.to be_a(Array) }

    it 'entries are of type ProjectGorgon::Shop::Log' do
      expect(today_shop_logs[0]).to be_a(ProjectGorgon::Shop::Log)
    end
  end

  describe "#content" do
    subject { player_log.content }

    it { is_expected.to be_a(String) }
  end
end
