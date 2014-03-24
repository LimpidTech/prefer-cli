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
  constructor: (@identifier, @prefer, configurator) ->
    @prefer.on 'updated', (@updatedConfigurator) =>
      @createHeader @updatedConfigurator, true
      @createFooter @updatedConfigurator, true

    sourceText = chalk.white configurator.state.source
    winston.debug 'Using ' + sourceText

    @configure configurator

  configure: (configurator) -> configurator.get (err, configuration) =>
    throw err if err

    @updatedConfigurator = undefined
    @initialize configurator, configuration

  selected: (configurator, keys, model) -> (err, selectedIndex) =>
    key = keys[selectedIndex]
    value = model[key]

    if _.isObject value
      @selections.push key
      @stack.push value
      @render configurator

  clean: ->
    # TODO: We can leverage #selections to reproduce #stack in #render
    @stack = []
    @selections = []
    @windows = [] unless @windows?

    window.detach() while window = @windows.pop()
    @header.detach() if @header?

  createHeader: (configurator, render) ->
    @header?.detach()
    return unless screen.height > 15

    content = """

      identifier: #{ chalk.white @identifier }
      location: #{ chalk.magenta configurator.state.source }

    """

    @header = blessed.box
      top: 0
      left:  5
      height: 4

    @header.setContent content
    screen.append @header

    screen.render() if render

  createFooter: (configurator, render) ->
    @statusLeft?.detach()
    @statusRight?.detach()
    @footer?.detach()
    return unless screen.height > 4

    changedFlag = chalk.red '[changed]' if @updatedConfigurator
    status = @selections.join '.'

    height = 1
    padding = 1

    @footer = blessed.box
      top: screen.height - height
      height: height

    screen.append @footer

    if changedFlag?
      rightWidth = changedFlag.length - padding

      @statusRight = blessed.box
        top: 0
        left: screen.width - rightWidth
        width: rightWidth
        height: height
        tags: yes
        content: "{right}#{ changedFlag }{/right}"

      @footer.append @statusRight

    rightWidth ?= 0

    @statusLeft = blessed.box
      top: 0
      left: 0
      width: screen.width - rightWidth
      height: height
      tags: yes
      content: status

    @footer.append @statusLeft

    screen.render() if render

  back: (configurator) -> =>
    return if @stack.length is 1

    @stack.pop()
    @selections.pop()

    currentWindow = @windows.pop()
    currentWindow.detach()

    newWindow = _.last @windows
    newWindow.focus()

    @render configurator

  backToTop: (configurator) -> => @back configurator while @stack.length > 1
  reset: => @configure @updatedConfigurator if @updatedConfigurator?

  render: (configurator) =>
    @stack.push _.cloneDeep @configuration unless @stack.length

    model = _.last @stack

    @createHeader configurator, model
    @createFooter configurator, model

    headerHeight = @header?.height or 0
    footerHeight = @footer?.height or 0

    window = blessed.list
      top: headerHeight
      left: 0
      height: screen.height - headerHeight - footerHeight
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

    window.key 't', @backToTop configurator
    window.key 'h', @back configurator
    window.key 'r', @reset

    window.on 'select', @selected configurator, keys, model

    window.focus()
    screen.render()

  initialize: (configurator, @configuration) ->
    @clean()
    @render configurator

  @main: ->
    yargs.demand 1
    {argv} = yargs

    if argv._.length is 0
      throw new Error '''
        A filename must be provided as the first command-line argument.
      '''

    configurationFileName = _.first argv._
    winston.debug 'Loading ' + chalk.white configurationFileName

    prefer.load configurationFileName, (err, configurator) ->
      throw err if err?
      new PreferCommandLineInterface configurationFileName, prefer, configurator


module.exports.main = PreferCommandLineInterface.main
