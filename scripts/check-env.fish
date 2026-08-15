#!/usr/bin/env fish

function check_cmd
    if command -q $argv[1]
        printf "%-12s %s\n" $argv[1] (command $argv[1] --version 2>/dev/null | head -n 1)
    else
        printf "%-12s MISSING\n" $argv[1]
    end
end

echo "Dark Project KD3B Control - environment check"
echo
check_cmd rustc
check_cmd cargo
check_cmd git
check_cmd opencode
check_cmd tshark
check_cmd wireshark
check_cmd node
check_cmd pnpm

echo
if test -e /etc/udev/rules.d/69-darkproject-legacy-195d.rules
    echo "udev rule: /etc/udev/rules.d/69-darkproject-legacy-195d.rules exists"
else
    echo "udev rule: project-specific 195d:2061 rule not found at the previously used path"
end
