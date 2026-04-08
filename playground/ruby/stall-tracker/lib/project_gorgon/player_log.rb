# @example
#   log = ProjectGorgon::PlayerLog.new("path/to/Player.log")
module ProjectGorgon
  class PlayerLog
    attr_reader :path

    TODAY_SHOP_LOGS_PATTERN = /^\[\d{2}:\d{2}:\d{2}\] LocalPlayer: ProcessBook\("Today's Shop Logs",/
    YESTERDAY_SHOP_LOGS_PATTERN = /^\[\d{2}:\d{2}:\d{2}\] LocalPlayer: ProcessBook\("Yesterday's Shop Logs",/

    # [01:27:17] LocalPlayer: ProcessBook("Shop Logs From 2
    DAYS_SHOP_LOGS_PATTERN = /^\[\d{2}:\d{2}:\d{2}\] LocalPlayer: ProcessBook\("Shop Logs From/

    # @param path [String]
    def initialize(path)
      @path = path
    end

    # @todo Parse login events
    # [19:55:34] Logged in as character Deradon. Time UTC=04/07/2026 19:55:34. Timezone Offset 02:00:00
    # def logged_in_as
    # end

    # @todo spec
    def all_shop_logs
      @all_shop_logs ||= today_shop_logs + yesterday_shop_logs + days_shop_logs
    end

    # @return [Array<ProjectGorgon::Shop::Log>]
    def today_shop_logs
      @today_shop_logs ||= content
        .lines
        .select { |line| line.match?(TODAY_SHOP_LOGS_PATTERN) }
        .map { |line| ProjectGorgon::Shop::Log.new(line) }
    end

    # @todo spec
    # @return [Array<ProjectGorgon::Shop::Log>]
    def yesterday_shop_logs
      @yesterday_shop_logs ||= content
        .lines
        .select { |line| line.match?(YESTERDAY_SHOP_LOGS_PATTERN) }
        .map { |line| ProjectGorgon::Shop::Log.new(line) }
    end

    # @todo spec
    # @return [Array<ProjectGorgon::Shop::Log>]
    def days_shop_logs
      @days_shop_logs ||= content
        .lines
        .select { |line| line.match?(DAYS_SHOP_LOGS_PATTERN) }
        .map { |line| ProjectGorgon::Shop::Log.new(line) }
    end

    # @return [String]
    def content
      @content ||= file.read
    end

    # @return [File]
    def file
      @file ||= File.open(path)
    end
  end
end
