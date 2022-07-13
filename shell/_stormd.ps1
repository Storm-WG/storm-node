
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'stormd' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'stormd'
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
        'stormd' {
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('--data-dir', 'data-dir', [CompletionResultType]::ParameterName, 'Data directory path')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Path for the configuration file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Path for the configuration file')
            [CompletionResult]::new('-M', 'M', [CompletionResultType]::ParameterName, 'ZMQ socket for peer message bus used to communicate with LNP node peerd service')
            [CompletionResult]::new('--msg', 'msg', [CompletionResultType]::ParameterName, 'ZMQ socket for peer message bus used to communicate with LNP node peerd service')
            [CompletionResult]::new('-X', 'X', [CompletionResultType]::ParameterName, 'ZMQ socket for internal service control bus')
            [CompletionResult]::new('--ctl', 'ctl', [CompletionResultType]::ParameterName, 'ZMQ socket for internal service control bus')
            [CompletionResult]::new('-R', 'R', [CompletionResultType]::ParameterName, 'ZMQ socket name/address for Storm Node client-server RPC API')
            [CompletionResult]::new('--rpc-endpoint', 'rpc-endpoint', [CompletionResultType]::ParameterName, 'ZMQ socket name/address for Storm Node client-server RPC API')
            [CompletionResult]::new('-E', 'E', [CompletionResultType]::ParameterName, 'ZMQ socket name/address for Storm extensions interface, used to handle application-specific messages to and from extension daemons, connected to this bus')
            [CompletionResult]::new('--ext-endpoint', 'ext-endpoint', [CompletionResultType]::ParameterName, 'ZMQ socket name/address for Storm extensions interface, used to handle application-specific messages to and from extension daemons, connected to this bus')
            [CompletionResult]::new('-S', 'S', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting storage daemon')
            [CompletionResult]::new('--store-endpoint', 'store-endpoint', [CompletionResultType]::ParameterName, 'ZMQ socket for connecting storage daemon')
            [CompletionResult]::new('-C', 'C', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('--chat-endpoint', 'chat-endpoint', [CompletionResultType]::ParameterName, 'ZMQ socket for chat daemon PUB/SUB API')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help information')
            [CompletionResult]::new('-V', 'V', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version information')
            [CompletionResult]::new('-v', 'v', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('--verbose', 'verbose', [CompletionResultType]::ParameterName, 'Set verbosity level')
            [CompletionResult]::new('-T', 'T', [CompletionResultType]::ParameterName, 'Spawn daemons as threads and not processes')
            [CompletionResult]::new('--threaded', 'threaded', [CompletionResultType]::ParameterName, 'Spawn daemons as threads and not processes')
            [CompletionResult]::new('--chat', 'chat', [CompletionResultType]::ParameterName, 'Run chat service')
            [CompletionResult]::new('--downpour', 'downpour', [CompletionResultType]::ParameterName, 'Run downpour (torrent-like) service')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
