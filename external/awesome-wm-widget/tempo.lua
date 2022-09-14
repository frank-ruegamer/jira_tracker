local wibox = require("wibox")
local watch = require("awful.widget.watch")

local tempo_widget = wibox.widget({align = "center", widget = wibox.widget.textbox})

local function update_tempo_status(_, stdout)
  tempo_widget:set_text(stdout)
end

local command = [[
  set -eo pipefail

  tracker=$(bash -c 'curl -s localhost:8000/tracker')
  key=$(jq -r .key <<< "${tracker}")
  description=$(jq -r --arg key "${key}" '.[] | select(.key == $key) | .fields.summary' ~/.jira/issues.json)
  jq -r --arg description "${description}" '"[" + .key + "] " + $description + ": " + .duration' <<< "${tracker}"
]]

local watch_command = string.format([[ bash -c "%s" ]], command:gsub('"', '\\"'))

watch(watch_command, 5, update_tempo_status)

return tempo_widget
