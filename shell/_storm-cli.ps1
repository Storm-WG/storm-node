
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
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('-L', 'L', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('--lnp', 'lnp', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('chat-listen', 'chat-listen', [CompletionResultType]::ParameterValue, 'chat-listen')
            [CompletionResult]::new('chat-send', 'chat-send', [CompletionResultType]::ParameterValue, 'chat-send')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'storm-cli;chat-listen' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('-L', 'L', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('--lnp', 'lnp', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'storm-cli;chat-send' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('-L', 'L', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('--lnp', 'lnp', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting LNP node RPC interface')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            break
        }
        'storm-cli;help' {
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
            [CompletionResult]::new('--rpc', 'rpc', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting Storm node RPC interface')
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
