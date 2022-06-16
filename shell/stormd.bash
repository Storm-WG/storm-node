_stormd() {
    local i cur prev opts cmds
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"
    cmd=""
    opts=""

    for i in ${COMP_WORDS[@]}
    do
        case "${i}" in
            "$1")
                cmd="stormd"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        stormd)
            opts="-h -V -v -d -m -x --help --version --verbose --data-dir --msg-endpoint --rpc-endpoint"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --data-dir)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --msg-endpoint)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -m)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --rpc-endpoint)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -x)
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

complete -F _stormd -o bashdefault -o default stormd
