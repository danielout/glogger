module ProjectGorgon
  require 'project_gorgon/collect_shop_log_items'
  require 'project_gorgon/create_csv'
  require 'project_gorgon/player_log'
  require 'project_gorgon/shop'

  class << self
    def write_sales_csv
      collector.items.each do |player, items|
        ProjectGorgon::CreateCsv.new(
          items.select(&:bought_action?),
          "#{player}-sales"
        ).write_csv
      end
    end

    def write_debug_csv
      collector.items.each do |player, items|
        ProjectGorgon::CreateCsv.new(
          items,
          "#{player}-debug"
        ).write_csv
      end
    end

    private

    def collector
      ProjectGorgon::CollectShopLogItems.new(
        [
          "/mnt/c/Users/derad/AppData/LocalLow/Elder\ Game/Project\ Gorgon/Player.log",
          "/mnt/c/Users/derad/AppData/LocalLow/Elder\ Game/Project\ Gorgon/Player-prev.log"
        ]
      )
    end
  end
end
