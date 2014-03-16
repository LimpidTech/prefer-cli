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
    @configurator.options.loader.on 'updated', =>
      @configurationUpdated = true
      @createHeader true
      @createFooter true

    sourceText = chalk.white configurator.options.results.source
    winston.debug 'Using ' + sourceText

    @configure()

  configure: -> @configurator.get (err, configuration) =>
    throw err if err

    @configurationUpdated = false
    @initialize configuration

  selected: (keys, model) -> (err, selectedIndex) =>
    key = keys[selectedIndex]
    value = model[key]

    if _.isObject value
      @selections.push key
      @stack.push value
      @render()

  clean: ->
    # TODO: We can leverage #selections to reproduce #stack in #render
    @stack = []
    @selections = []
    @windows = [] unless @windows?

    window.detach() while window = @windows.pop()
    @header.detach() if @header?

  createHeader: (render) ->
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

    screen.render() if render

  createFooter: (render) ->
    @statusLeft?.detach()
    @statusRight?.detach()
    @footer?.detach()
    return unless screen.height > 4

    changedFlag = chalk.red '[changed]' if @configurationUpdated
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

  initialize: (@configuration) ->
    @clean()
    @render()

  back: =>
    return if @stack.length is 1

    @stack.pop()
    @selections.pop()

    currentWindow = @windows.pop()
    currentWindow.detach()

    newWindow = _.last @windows
    newWindow.focus()

    @render()

  backToTop: => @back() while @stack.length > 1
  reset: => @configure() if @configurationUpdated

  render: =>
    @stack.push _.cloneDeep @configuration unless @stack.length

    model = _.last @stack

    @createHeader model
    @createFooter model

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

    window.key 't', @backToTop
    window.key 'h', @back
    window.key 'r', @reset

    window.on 'select', @selected keys, model

    window.focus()
    screen.render()

  @main: ->
    yargs.demand 1
    {argv} = yargs

    configurationFileName = _.first argv._
    winston.debug 'Loading ' + chalk.white configurationFileName

    prefer.load configurationFileName, (err, configurator) ->
      throw err if err?
      new PreferCommandLineInterface configurationFileName, configurator


module.exports.main = PreferCommandLineInterface.main
