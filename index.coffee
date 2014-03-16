_ = require 'lodash'
blessed = require 'blessed'
chalk = require 'chalk'
prefer = require 'prefer'
winston = require 'winston'
yargs = require 'yargs'


program = blessed.program()

screen = blessed.screen
  dump: true

screen.key ['escape', 'C-c', 'q'], -> process.exit 0


class PreferCommandLineInterface
  constructor: (@identifier, @configurator) ->
    sourceText = chalk.white configurator.options.results.source
    winston.debug 'Using ' + sourceText

    @configure()

  configure: ->
    @configurator.get (err, configuration) =>
      throw err if err
      @initialize configuration

  selected: (keys, model) -> (err, selectedIndex) =>
    value = model[keys[selectedIndex]]

    if _.isObject value
      @stack.push value
      @render()

  back: =>
    return if @stack.length is 1

    @stack.pop()

    currentWindow = @windows.pop()
    currentWindow.detach()

    newWindow = _.last @windows
    newWindow.focus()

    @render()

  backToTop: =>
    @back() while @stack.length > 1

  render: =>
    @stack.push @configuration unless @stack.length
    model = _.last @stack

    top = @header?.height or 0

    window = blessed.list
      top: top
      left: 0
      height: screen.lines - top
      itemFg: 'cyan'
      selectedFg: 'white'
      selectedBg: 'blue'
      keys: 'vi'
      mouse: true
      vi: true

    @windows.push window
    screen.append window

    keys = _.keys model

    for key in keys
      value = model[key]
      typeText = chalk.blue typeof value

      if _.isObject value
        valueText = ''
      else
        valueText = chalk.magenta value.toString()

      nameText = chalk.white key

      window.add "#{ nameText } = [#{ typeText }] #{ valueText }"

    window.key 't', @backToTop
    window.key 'h', @back
    window.on 'select', @selected keys, model

    window.focus()
    screen.render()

  clean: ->
    window.detach() while window = @windows.pop() if @windows?
    @header.detach() if @header?

  createHeader: ->
    @header?.detach()
    return unless screen.height > 15

    content = """

      identifier: #{ chalk.white @identifier }
      location: #{ chalk.magenta @configurator.options.results.source }

    """

    @header = blessed.box
      top: 0
      left: 5
      height: 4

    @header.setContent content
    screen.append @header

  initialize: (@configuration) ->
    @clean()

    @stack = []
    @windows = []

    @createHeader()
    @render()

  @main: ->
    yargs.demand 1
    {argv} = yargs

    configurationFileName = _.first argv._
    winston.debug 'Loading ' + chalk.white configurationFileName

    prefer.load configurationFileName, (err, configurator) ->
      throw err if err?
      new PreferCommandLineInterface configurationFileName, configurator


module.exports.main = PreferCommandLineInterface.main
