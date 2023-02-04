set -l _backup_commands clean list help restore the versions
set -l _backup_edit_commands edit rm

function _backup_versions
    set -l TARGET ( \
        commandline \
        | grep -oP 'backup\s+\w+\s*\K(.*)\s+(--version=|-v=)' \
        | sed -e 's: \+--version=::g;s: \+-v=::g'\
    )
    backup versions "$TARGET" | awk '{ print $1 }'
end

complete -f -c backup \
    -n "not __fish_seen_subcommand_from $_backup_commands" \
    -a help

complete -f -c backup \
    -n "not __fish_seen_subcommand_from $_backup_commands" \
    -a list \
    -d "list current backups"

complete -f -c backup \
    -n "not __fish_seen_subcommand_from $_backup_commands" \
    -a versions \
    -d "list backup versions for given file/directory"

complete -c backup \
    -n "not __fish_seen_subcommand_from $_backup_commands" \
    -a the \
    -d "backup provided file/directory"

complete -f -c backup \
    -n "not __fish_seen_subcommand_from $_backup_commands" \
    -a clean \
    -d "clean backups"

complete -f -c backup \
    -n "not __fish_seen_subcommand_from $_backup_commands" \
    -a restore \
    -d "restore backup"

complete -f -c backup \
    -n "__fish_seen_subcommand_from versions clean restore" \
    -a "(backup list; and echo .)"

complete -f -c backup \
    -n "__fish_seen_subcommand_from clean restore" \
    -s "v" \
    -l "version" \
    -a "(_backup_versions)"
