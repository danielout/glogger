class ProjectGorgon::Shop::Log
  LINE_PATTERN = /
    \A
    \[(?<time>\d{2}:\d{2}:\d{2})\]\s+  # [20:26:42]
    (?<source>\w+):                    # LocalPlayer:
    \s+
    (?<method>\w+)                     # ProcessBook
    \((?<args>.*)\)                    # ("Today's Shop Logs", ...)
    \Z
  /mx

  ARGS_PATTERN = /
    "(?<title>.*?)",\s+                # Today's Shop Logs
    "(?<body>.*)",\s+                  # Tue Apr 16:25 - ...
    "(?<book_type>.*?)",\s+            # PlayerShopLog
    "(?<arg4>.*?)",\s+
    "(?<arg5>.*?)",\s+
    (?<flag1>False|True),\s+
    (?<flag2>False|True),\s+
    (?<flag3>False|True),\s+
    (?<flag4>False|True),\s+
    (?<flag5>False|True),\s+
    "(?<arg11>.*?)"
  /mx

  ENTRY_REGEX = /
    (
      [A-Z][a-z]{1,3}\s           # Tue
      [A-Z][a-z]{1,3}\s           # Apr
      \d{1,2}\s                   # 7
      \d{2}:\d{2}                 # 16:25
      .*?                         # everything after it, lazily
    )
    (?=
      [A-Z][a-z]{1,3}\s
      [A-Z][a-z]{1,3}\s
      \d{1,2}\s
      \d{2}:\d{2}                 # next timestamp
      |
      \n\n",\s*"PlayerShopLog"    # end of ProcessBook text
    )
  /mx

  attr_reader :line

  # @param line [String]
  def initialize(line)
    @line = line
  end

  def time = line_match[:time]
  def source = line_match[:source]
  def method = line_match[:method]
  def title = args[:title]

  # @return [Array<ProjectGorgon::Shop::LogMessage::Base>]
  def log_messages
    @log_messages ||= entries.each_with_index.map do |item, index|
      ProjectGorgon::Shop::LogMessage.for(item:, index:)
    end
  end

  # @return [Array<String>] Log entries, sorted by oldest to newest
  def entries
    @entries ||= args[:body]
      .scan(ENTRY_REGEX)
      .flatten
      .map(&:strip)
      .reverse
  end

  # @return [String, nil]
  # @note Another, more consistent way to identify current logged in character
  #       is to parse "Logged in as" events
  # @see {ProjectGorgon::PlayerLog#logged_in_as}
  def owner
    log_messages.find(&:owner_action?)&.player
  end

  # @return [MatchData]
  def args
    @args ||= sanitized_args.match(ARGS_PATTERN)
  end

  # @return [String]
  def sanitized_args
    raw_args
      .gsub('\\n', "\n")
      .gsub("\n\n", "\n")
      .gsub("\\\"", "\"")
  end

  # @return [String]
  def raw_args = line_match[:args]

  # @return [MatchData]
  def line_match
    @line_match ||= line.strip.match(LINE_PATTERN)
  end
end
