# @example
#   collector = ProjectGorgon::CollectShopLogItems.new(
#     [
#       "path/to/Player.log",
#       "path/to/Player-prev.log",
#     ]
#   )
#   collector.items => {
#     "Deradon" => ...,
#     "Wogan" => ...,
#   }
module ProjectGorgon
  # @todo Rename to `CollectShopLogMessages`
  class CollectShopLogItems
    attr_reader :paths

    def initialize(paths)
      @paths = paths
    end

    def items
      return @items if defined?(@items)

      @items = Hash.new { |hash, player| hash[player] = Set.new }

      shop_logs.each do |shop_log|
        shop_log.log_messages.each { |item| @items[shop_log.owner].add(item) }
      end

      @items.transform_values! { |set| set.to_a.sort_by(&:date) }
    end

    # @return [Array<ProjectGorgon::ShopLog>]
    # @todo spec
    def shop_logs
      @shop_logs ||= player_logs.map(&:all_shop_logs).flatten
    end

    # @return [Array<ProjectGorgon::PlayerLog>]
    def player_logs
      @player_logs ||= paths.map { |path| ProjectGorgon::PlayerLog.new(path) }
    end
  end
end
