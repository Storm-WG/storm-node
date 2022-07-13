
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'storm-cli' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'storm-cli'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'storm-cli' {
            [CompletionResult]::new('-S', 'S', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--storm', 'storm', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--store', 'store', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting storage daemon')
            [CompletionResult]::new('-C', 'C', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('--chat', 'chat', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('-L', 'L', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('--lnp', 'lnp', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('chat-listen', 'chat-listen', [CompletionResultType]::ParameterValue, 'Listen for the incoming chat messages from a remote peer')
            [CompletionResult]::new('chat-send', 'chat-send', [CompletionResultType]::ParameterValue, 'Send typed-in messages to another peer')
            [CompletionResult]::new('file-containerize', 'file-containerize', [CompletionResultType]::ParameterValue, 'Convert on-disk file into a container in the Store database')
            [CompletionResult]::new('file-assemble', 'file-assemble', [CompletionResultType]::ParameterValue, 'Assemble a file from a Store database-present container and save as a file')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'storm-cli;chat-listen' {
            [CompletionResult]::new('--connect', 'connect', [CompletionResultType]::ParameterName, 'Remote node address to force connection (re)establishment')
            [CompletionResult]::new('-S', 'S', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--storm', 'storm', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--store', 'store', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting storage daemon')
            [CompletionResult]::new('-C', 'C', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('--chat', 'chat', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('-L', 'L', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('--lnp', 'lnp', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'storm-cli;chat-send' {
            [CompletionResult]::new('--connect', 'connect', [CompletionResultType]::ParameterName, 'Remote node address to force connection (re)establishment')
            [CompletionResult]::new('-S', 'S', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--storm', 'storm', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--store', 'store', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting storage daemon')
            [CompletionResult]::new('-C', 'C', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('--chat', 'chat', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('-L', 'L', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('--lnp', 'lnp', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'storm-cli;file-containerize' {
            [CompletionResult]::new('-m', 'm', [CompletionResultType]::ParameterName, 'MIME file type')
            [CompletionResult]::new('--mime', 'mime', [CompletionResultType]::ParameterName, 'MIME file type')
            [CompletionResult]::new('-S', 'S', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--storm', 'storm', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--store', 'store', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting storage daemon')
            [CompletionResult]::new('-C', 'C', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('--chat', 'chat', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('-L', 'L', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('--lnp', 'lnp', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'storm-cli;file-assemble' {
            [CompletionResult]::new('-S', 'S', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--storm', 'storm', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--store', 'store', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting storage daemon')
            [CompletionResult]::new('-C', 'C', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('--chat', 'chat', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('-L', 'L', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('--lnp', 'lnp', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'storm-cli;help' {
            [CompletionResult]::new('-S', 'S', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--storm', 'storm', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--store', 'store', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting storage daemon')
            [CompletionResult]::new('-C', 'C', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('--chat', 'chat', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('-L', 'L', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('--lnp', 'lnp', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
