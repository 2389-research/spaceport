# ABOUTME: Collects bounded listener and environment discovery in one host shell.
# ABOUTME: Keeps process attribution in one awk program and batches fallback ps requests.
for herdr_dir in /usr/sbin /sbin; do
    case ":$PATH:" in *":$herdr_dir:"*) ;; *) PATH="$PATH:$herdr_dir" ;; esac
done
if command -v ss >/dev/null 2>&1; then
    herdr_ports=$(ss -ltnp 2>/dev/null | awk '{
        rest = $0
        while (match(rest, /\("[^"]*",pid=[0-9]+/)) {
            entry = substr(rest, RSTART + 2, RLENGTH - 2)
            rest = substr(rest, RSTART + RLENGTH)
            split(entry, part, "\",pid=")
            print "L", part[2], $4, part[1]
            if (++count == 1024) exit
        }
    }')
elif command -v lsof >/dev/null 2>&1; then
    herdr_ports=$(lsof -nP -iTCP -sTCP:LISTEN -Fpcn 2>/dev/null | awk '
        /^p/ { pid = substr($0, 2) }
        /^c/ { name = substr($0, 2) }
        /^n/ { print "L", pid, substr($0, 2), name; if (++count == 1024) exit }')
else
    herdr_ports=N
fi
printf '%s\n' "$herdr_ports"
herdr_pids=$(printf '%s\n' "$herdr_ports" | awk '$1 == "L" && $2 ~ /^[0-9]+$/ { print $2 }' | sort -un | head -n 128)
herdr_owner_awk='
function emit() {
    if (pid !~ /^[0-9]+$/ || workspace == "" || length(workspace) > 64 || workspace !~ /^[!-~]+$/ || index(workspace, "=")) return
    if (length(socket) > 1024 || socket ~ /[[:cntrl:]]/) socket = ""
    printf "E %s %s %s\n", pid, workspace, socket
}
function value(text, name, position, rest) {
    text = " " text
    position = index(text, " " name "=")
    if (!position) return ""
    rest = substr(text, position + length(name) + 2)
    if (match(rest, / [A-Za-z_][A-Za-z0-9_]*=/)) rest = substr(rest, 1, RSTART - 1)
    return rest
}
function take(record) {
    if (!have_workspace && index(record, "HERDR_WORKSPACE_ID=") == 1) { workspace = substr(record, 20); have_workspace = 1 }
    if (!have_socket && index(record, "HERDR_SOCKET_PATH=") == 1) { socket = substr(record, 19); have_socket = 1 }
}
BEGIN {
    if (mode == "proc") {
        RS = "\0"
        for (i = 1; i < ARGC; i++) {
            file = ARGV[i]
            n = split(file, path, "/")
            pid = path[n - 1]
            workspace = socket = ""
            have_workspace = have_socket = 0
            while ((getline record < file) > 0) take(record)
            close(file)
            emit()
        }
        exit
    }
    if (mode == "lines") pid = owner_pid
    if (mode == "ps") { count = split(requested, ids, ","); for (i = 1; i <= count; i++) allowed[ids[i]] = 1 }
}
mode == "lines" { take($0); next }

mode == "ps" && ($1 in allowed) {
    pid = $1
    workspace = value($0, "HERDR_WORKSPACE_ID")
    socket = value($0, "HERDR_SOCKET_PATH")
    emit()
}
END { if (mode == "lines") emit() }
'
set --
herdr_ps_pids=
for herdr_pid in $herdr_pids; do
    if [ -r "/proc/$herdr_pid/environ" ]; then
        set -- "$@" "/proc/$herdr_pid/environ"
    else
        herdr_ps_pids="${herdr_ps_pids:+$herdr_ps_pids,}$herdr_pid"
    fi
done
if [ "$#" -gt 0 ]; then
    if [ -z "${herdr_scan_nul_awk-}" ]; then
        herdr_scan_nul_awk=$(printf 'first\000second\000' | awk 'BEGIN { RS="\0" } END { print NR }')
    fi
    if [ "$herdr_scan_nul_awk" = 2 ]; then
        LC_ALL=C awk -v mode=proc "$herdr_owner_awk" "$@" 2>/dev/null
    else
        for herdr_file; do
            herdr_pid=${herdr_file%/environ}
            herdr_pid=${herdr_pid##*/}
            tr '\0' '\n' < "$herdr_file" 2>/dev/null | LC_ALL=C awk -v mode=lines -v owner_pid="$herdr_pid" "$herdr_owner_awk"
        done
    fi
fi
if [ -n "$herdr_ps_pids" ]; then
    ps eww -o pid= -o command= -p "$herdr_ps_pids" 2>/dev/null | LC_ALL=C awk -v mode=ps -v requested="$herdr_ps_pids" "$herdr_owner_awk"
fi
true
