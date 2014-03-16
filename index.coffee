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
  constructor: (@configurator) ->
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

  render: =>
    program.clear()

    @stack.push @configuration unless @stack.length
    model = _.last @stack

    window = blessed.list
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
      window.add key + '[' + typeof value + ']' + ' = ' + value.toString()

    window.key 'h', @back

    window.on 'select', @selected keys, model

    window.focus()
    screen.render()

  initialize: (@configuration) ->
    window.detach() while window = @windows.pop() if @windows?

    @stack = []
    @windows = []

    @render()

  @main: ->
    yargs.demand 1
    {argv} = yargs

    configurationFileName = _.first argv._
    # winston.debug 'Loading ' + chalk.white configurationFileName

    prefer.load configurationFileName, (err, configurator) ->
      throw err if err?
      new PreferCommandLineInterface configurator


module.exports.main = PreferCommandLineInterface.main
