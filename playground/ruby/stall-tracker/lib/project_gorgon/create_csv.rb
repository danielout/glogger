require 'csv'

# @example Writing a CSV with all all items bought from the shop
#   collector = ProjectGorgon::CollectShopLogItems.new(
#     [
#       "path/to/Player.log",
#       "path/to/Player-prev.log",
#     ]
#   )
#
#   ProjectGorgon::CreateCsv.new(
#     collector.items["Deradon"].select(&:bought_action),
#     "deradon-all-bought"
#   ).write_csv
#
# @example Writing a CSV with all shop activity
#   collector = ProjectGorgon::CollectShopLogItems.new(
#     [
#       "path/to/Player.log",
#       "path/to/Player-prev.log",
#     ]
#   )
#
#   ProjectGorgon::CreateCsv.new(
#     collector.items["Wogan"],
#     "wogan-debug"
#   ).write_csv
module ProjectGorgon
  class CreateCsv
    CSV_HEADERS = %w[
      date
      player
      action
      item
      price_unit
      quantity
      price_total
    ].freeze

    attr_reader :messages, :name

    # @param messages [Array<ProjectGorgon::Shop::LogMessage::Base>]
    def initialize(messages, name)
      @messages = messages
      @name = name
    end

    def write_csv
      CSV.open("#{name}.csv", "w", write_headers: true, headers: CSV_HEADERS) do |csv|
        messages.each do |item|
          csv << item.to_csv_row
        end
      end
    end
  end
end
