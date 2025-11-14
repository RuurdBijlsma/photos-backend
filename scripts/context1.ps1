#Requires -Version 5.1

<#
.SYNOPSIS
    Generates a comprehensive context for Large Language Models (LLMs) to help explain programming problems in a Rust repository.

.DESCRIPTION
    This script automates the collection of project information to create a detailed context for an LLM.
    It gathers:
    - A file and directory tree structure.
    - Content of files selected via the clipboard.
    - Contents of all Cargo.toml files.
    - The output of 'git diff' for uncommitted changes.
    - An OpenAPI specification from a configured URL.
    - SQL migration files.

    The final context is placed on the clipboard. The script is designed to be run from the root of a Git repository.
#>

# --- Configuration ---
# Set the URL for your project's OpenAPI JSON specification.
# If this is an empty string (""), the script will skip the OpenAPI prompt.
$openApiUrl = ""

# Add patterns to ignore beyond what's in .gitignore.
# These use the .gitignore pattern format.
$additionalIgnorePatterns = @(
    ".sqlx/"
    "*.log"
    "media_dir/"
)
# --- End of Configuration ---

# --- Script Initialisation ---
try
{
    # Determine the project root, assuming the script is in a 'scripts' subdirectory.
    $projectRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
    Set-Location $projectRoot

    Write-Host "Project root set to: $projectRoot"
}
catch
{
    Write-Error "Error: Could not determine project root. Please run this script from a 'scripts' subdirectory of your project."
    exit 1
}

# --- Helper Functions ---

Function Read-YesNoPrompt($prompt, [ValidateSet('y', 'n')]$default)
{
    $options = switch ($default)
    {
        'y' {
            '(Y/n)'
        }
        'n' {
            '(y/N)'
        }
    }
    $response = Read-Host -Prompt "$prompt $options"
    if ( [string]::IsNullOrWhiteSpace($response))
    {
        return $default
    }
    return $response.ToLower()
}

Function Get-GitIgnorePatterns($rootPath)
{
    $gitignorePath = Join-Path $rootPath ".gitignore"
    if (Test-Path $gitignorePath)
    {
        return Get-Content $gitignorePath | Where-Object { $_ -and $_ -notlike '#*' }
    }
    return @()
}

Function Test-PathAgainstPatterns($path, $patterns, $rootPath)
{
    # Get the relative path from the project root, using forward slashes for consistency.
    $relativePath = $path.Substring($rootPath.Length).Replace('\', '/')
    if ( $relativePath.StartsWith('/'))
    {
        $relativePath = $relativePath.Substring(1)
    }

    foreach ($rawPattern in $patterns)
    {
        $pattern = $rawPattern.Trim()
        if ([string]::IsNullOrWhiteSpace($pattern) -or $pattern.StartsWith('#'))
        {
            continue
        }

        # Check if the gitignore pattern is rooted to the project directory (starts with '/')
        $isRooted = $pattern.StartsWith('/')

        # Prepare the core of the pattern for regex conversion.
        # This removes the special gitignore slashes.
        $corePattern = $pattern.TrimStart('/').TrimEnd('/')

        # Convert the core pattern into a regex, escaping special characters
        # and then translating the wildcards.
        $regex = [regex]::Escape($corePattern).Replace('\*', '.*').Replace('\?', '.')

        # Build the final regex based on whether the pattern is rooted or not.
        $finalRegex = ""
        if ($isRooted)
        {
            # The pattern is rooted, so it MUST match from the start of the path.
            # It should match the pattern itself, followed by the end of the line ($) or a slash (/).
            $finalRegex = '^' + $regex + '($|/)'
        }
        else
        {
            # The pattern can be anywhere. It must be a complete path segment.
            # This means it's preceded by the start of the line or a slash,
            # and followed by the end of the line or a slash.
            $finalRegex = '(^|/)' + $regex + '($|/)'
        }

        if ($relativePath -match $finalRegex)
        {
            # A match was found, so this path should be ignored.
            return $true
        }
    }

    # If no patterns matched, the path should not be ignored.
    return $false
}

Function Get-Tree(
        [string]$path,
        [string]$indent,
        [string[]]$ignorePatterns,
        [string]$rootPath
)
{
    $items = Get-ChildItem -Path $path | Sort-Object -Property { $_.PSIsContainer -eq $false }, Name

    for ($i = 0; $i -lt $items.Count; $i++) {
        $item = $items[$i]

        if (Test-PathAgainstPatterns -path $item.FullName -patterns $ignorePatterns -rootPath $rootPath)
        {
            continue
        }

        $isLast = ($i -eq $items.Count - 1)
        $marker = if ($isLast)
        {
            "\-- "
        }
        else
        {
            "+-- "
        }
        $output = "$indent$marker$( $item.Name )"
        $output

        if ($item.PSIsContainer)
        {
            # CORRECTED: Using a subexpression '$(...)' to ensure the 'if' is evaluated correctly.
            $newIndent = $indent + $( if ($isLast)
            {
                "    "
            }
            else
            {
                "|   "
            } )
            Get-Tree -path $item.FullName -indent $newIndent -ignorePatterns $ignorePatterns -rootPath $rootPath
        }
    }
}


# --- Main Logic ---

$ErrorActionPreference = 'Stop'
$contextBuilder = New-Object System.Text.StringBuilder

# 1. Load Ignore Patterns
$ignorePatterns = (Get-GitIgnorePatterns -rootPath $projectRoot) + $additionalIgnorePatterns
Write-Host "Loaded $( @($ignorePatterns).Length ) ignore patterns from .gitignore and configuration."

# 1.5: Add a section to list the ignore patterns being used.
$contextBuilder.AppendLine("## Ignored File & Directory Patterns") | Out-Null
$contextBuilder.AppendLine("The following patterns have been excluded from the file tree below:") | Out-Null
$contextBuilder.AppendLine('```') | Out-Null
# Loop through the combined patterns and add them to the context
foreach ($pattern in $ignorePatterns) {
    $contextBuilder.AppendLine($pattern) | Out-Null
}
$contextBuilder.AppendLine('```') | Out-Null
$contextBuilder.AppendLine() | Out-Null

# 2. Generate and add the file tree
$contextBuilder.AppendLine("## File & Directory Structure") | Out-Null
$contextBuilder.AppendLine('```') | Out-Null
$contextBuilder.AppendLine(".") | Out-Null
$treeOutput = Get-Tree -path $projectRoot -indent "" -ignorePatterns $ignorePatterns -rootPath $projectRoot
$contextBuilder.Append($treeOutput -join [Environment]::NewLine) | Out-Null
$contextBuilder.AppendLine() | Out-Null # Add a newline for cleaner separation
$contextBuilder.AppendLine('```') | Out-Null
$contextBuilder.AppendLine() | Out-Null

# 3. Process files from clipboard
if (Get-Clipboard -Format FileDropList -ErrorAction SilentlyContinue)
{
    $clipboardFiles = Get-Clipboard -Format FileDropList
    Write-Host "`nAdding $( $clipboardFiles.Count ) file(s) from the clipboard:" -ForegroundColor Yellow
    $clipboardFiles | ForEach-Object { Write-Host "- $( $_.Name )" -ForegroundColor Green }

    foreach ($file in $clipboardFiles)
    {
        if (Test-Path $file.FullName)
        {
            $relativePath = $file.FullName.Substring($projectRoot.Length + 1)
            $contextBuilder.AppendLine("## File: $relativePath") | Out-Null
            $contextBuilder.AppendLine('```') | Out-Null
            $contextBuilder.AppendLine((Get-Content $file.FullName -Raw)) | Out-Null
            $contextBuilder.AppendLine('```') | Out-Null
            $contextBuilder.AppendLine() | Out-Null
        }
    }
}

# 4. Add Cargo.toml files
$cargoFiles = Get-ChildItem -Path $projectRoot -Recurse -Filter "Cargo.toml" | Where-Object { -not (Test-PathAgainstPatterns $_.FullName $ignorePatterns $projectRoot) }
if ($cargoFiles)
{
    if ((Read-YesNoPrompt -prompt "`nFound $( $cargoFiles.Count ) Cargo.toml file(s). Add all of them?" -default 'y') -eq 'y')
    {
        foreach ($file in $cargoFiles)
        {
            $relativePath = $file.FullName.Substring($projectRoot.Length + 1)
            $contextBuilder.AppendLine("## File: $relativePath") | Out-Null
            $contextBuilder.AppendLine('```toml') | Out-Null
            $contextBuilder.AppendLine((Get-Content $file.FullName -Raw)) | Out-Null
            $contextBuilder.AppendLine('```') | Out-Null
            $contextBuilder.AppendLine() | Out-Null
            Write-Host "Added content of $relativePath" -ForegroundColor Green
        }
    }
}

# 5. Add Git Diff
if ((Read-YesNoPrompt -prompt "`nAdd uncommitted changes (git diff)?" -default 'n') -eq 'y')
{
    $gitDiff = git diff
    if ($gitDiff)
    {
        $contextBuilder.AppendLine("## Git Diff (Uncommitted Changes)") | Out-Null
        $contextBuilder.AppendLine('```diff') | Out-Null
        $contextBuilder.AppendLine($gitDiff) | Out-Null
        $contextBuilder.AppendLine('```') | Out-Null
        $contextBuilder.AppendLine() | Out-Null
        Write-Host "Added git diff." -ForegroundColor Green
    }
    else
    {
        Write-Host "No uncommitted changes to add."
    }
}

# 6. Add OpenAPI Spec
if (-not [string]::IsNullOrWhiteSpace($openApiUrl))
{
    if ((Read-YesNoPrompt -prompt "`nFetch and add OpenAPI spec from $openApiUrl?" -default 'y') -eq 'y')
    {
        try
        {
            Write-Host "Fetching OpenAPI spec..."
            $apiSpec = Invoke-RestMethod -Uri $openApiUrl
            $apiSpecJson = $apiSpec | ConvertTo-Json -Depth 10
            $contextBuilder.AppendLine("## OpenAPI Specification") | Out-Null
            $contextBuilder.AppendLine('```json') | Out-Null
            $contextBuilder.AppendLine($apiSpecJson) | Out-Null
            $contextBuilder.AppendLine('```') | Out-Null
            $contextBuilder.AppendLine() | Out-Null
            Write-Host "Added OpenAPI spec." -ForegroundColor Green
        }
        catch
        {
            Write-Warning "Failed to fetch OpenAPI spec: $_"
        }
    }
}

# 7. Add SQL Migrations
$migrationsPath = Join-Path $projectRoot "migrations"
if (Test-Path $migrationsPath)
{
    $sqlFiles = Get-ChildItem -Path $migrationsPath -Filter "*.sql"
    if ($sqlFiles)
    {
        if ((Read-YesNoPrompt -prompt "`nFound $( $sqlFiles.Count ) SQL migration file(s). Add all of them?" -default 'y') -eq 'y')
        {
            $contextBuilder.AppendLine("## SQL Migrations") | Out-Null
            foreach ($file in $sqlFiles)
            {
                $contextBuilder.AppendLine("### File: $( $file.Name )") | Out-Null
                $contextBuilder.AppendLine('```sql') | Out-Null
                $contextBuilder.AppendLine((Get-Content $file.FullName -Raw)) | Out-Null
                $contextBuilder.AppendLine('```') | Out-Null
                $contextBuilder.AppendLine() | Out-Null
                Write-Host "Added content of $( $file.Name )" -ForegroundColor Green
            }
        }
    }
}

# --- Finalisation ---
$finalContext = $contextBuilder.ToString()
$finalContext | Set-Clipboard

Write-Host "`n✅ Success! The context has been generated and copied to your clipboard." -ForegroundColor Cyan