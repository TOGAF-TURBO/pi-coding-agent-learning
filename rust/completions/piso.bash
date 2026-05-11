_piso() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="piso"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        piso)
            opts="-m -c -r -n -v -p -e -h -V --provider --model --api-key --base-url --thinking --system-prompt --append-system-prompt --continue --resume --fork --session --mode --no-session --no-tools --no-builtin-tools --no-extensions --no-skills --no-prompt-templates --no-context-files --list-models --list-sessions --offline --verbose --print --export --extension --help --version [MESSAGES]..."
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --provider)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --model)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -m)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --api-key)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --base-url)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --thinking)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --system-prompt)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --append-system-prompt)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --fork)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --session)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --mode)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --list-models)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --export)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --extension)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -e)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _piso -o nosort -o bashdefault -o default piso
else
    complete -F _piso -o bashdefault -o default piso
fi
