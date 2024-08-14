#!/bin/bash

# Function to extract changes for a given version from the changelog
extract_changes() {

    local version="$1"
    local start_pattern="^## \[${version}\] \- [0-9]{4}\-[0-9]{2}\-[0-9]{2}$"
    local end_pattern="^##.*$"
    local in_section=0
    local changes=""


    while IFS= read -r line; do

        if (( in_section )); then

            if [[ $line =~ $end_pattern ]]; then
                in_section=0
                break
            fi
            changes+="$line\n"

        else 
            if [[ $line =~ $start_pattern ]]; then
                in_section=1
                changes+="$line\n"
                continue
            fi
        fi
    done < "CHANGELOG.md"
    printf "%b" "$changes" > release-note.md
}

# Main function
main() {
    local version

    if [[ $# -ne 1 ]]; then
        printf "Usage: %s <version>\n" "$0" >&2
        return 1
    fi

    if [[ ! -f "CHANGELOG.md" ]]; then
        printf "Changelog file not found: %s\n" "$changelog" >&2
        return 1
    fi

    version="$1"

    if ! extract_changes "$version"; then
        printf "Failed to extract changes for version: %s\n" "$version" >&2
        return 1
    fi
}

main "$@"
