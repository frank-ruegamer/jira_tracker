local wibox = require("wibox")
local watch = require("awful.widget.watch")

local tempo_widget = wibox.widget({align = "center", widget = wibox.widget.textbox})

local function update_tempo_status(_, stdout)
  tempo_widget:set_text(stdout)
end

watch("bash -c '$HOME/.config/awesome/widgets/tempo-tracker/tracker.bash'", 5, update_tempo_status)

return tempo_widget
