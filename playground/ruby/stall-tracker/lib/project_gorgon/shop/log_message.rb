module ProjectGorgon::Shop::LogMessage
  require 'project_gorgon/shop/log_message/base'

  require 'project_gorgon/shop/log_message/added'
  require 'project_gorgon/shop/log_message/bought'
  require 'project_gorgon/shop/log_message/collected'
  require 'project_gorgon/shop/log_message/configured'
  require 'project_gorgon/shop/log_message/made_visible'
  require 'project_gorgon/shop/log_message/removed'
  require 'project_gorgon/shop/log_message/unknown'

  MESSAGE_TYPES = [
    ProjectGorgon::Shop::LogMessage::Added,
    ProjectGorgon::Shop::LogMessage::Bought,
    ProjectGorgon::Shop::LogMessage::Collected,
    ProjectGorgon::Shop::LogMessage::Configured,
    ProjectGorgon::Shop::LogMessage::MadeVisible,
    ProjectGorgon::Shop::LogMessage::Removed
  ].freeze

  class << self
    # @return [ProjectGorgon::Shop::LogMessage::Base]
    def for(item:, index:)
      MESSAGE_TYPES.each do |type|
        match = type.match(item, index)

        return match if match
      end

      ProjectGorgon::Shop::LogMessage::Unknown.new(body: item, index:)
    end
  end
end
