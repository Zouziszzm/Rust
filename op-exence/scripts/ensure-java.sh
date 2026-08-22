#!/usr/bin/env bash
set -euo pipefail

# Find Java 21 for Spring Boot / Gradle (macOS Homebrew or java_home).

find_java_home() {
  if command -v /usr/libexec/java_home >/dev/null 2>&1; then
    local home
    home=$(/usr/libexec/java_home -v 21 2>/dev/null || true)
    if [[ -n "$home" && -x "$home/bin/java" ]]; then
      echo "$home"
      return 0
    fi
  fi

  local brew_prefix
  if command -v brew >/dev/null 2>&1; then
    brew_prefix=$(brew --prefix openjdk@21 2>/dev/null || true)
    if [[ -n "$brew_prefix" && -x "$brew_prefix/bin/java" ]]; then
      echo "$brew_prefix"
      return 0
    fi
    if [[ -d "$brew_prefix/libexec/openjdk.jdk/Contents/Home/bin/java" ]]; then
      echo "$brew_prefix/libexec/openjdk.jdk/Contents/Home"
      return 0
    fi
  fi

  for candidate in \
    /opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home \
    /usr/local/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home; do
    if [[ -x "$candidate/bin/java" ]]; then
      echo "$candidate"
      return 0
    fi
  done

  return 1
}

if JAVA_HOME=$(find_java_home); then
  export JAVA_HOME
  export PATH="$JAVA_HOME/bin:$PATH"
else
  echo "Java 21 is required for the Spring Boot frontend." >&2
  echo "" >&2
  echo "Install:" >&2
  echo "  brew install openjdk@21" >&2
  echo "" >&2
  echo "Then re-run:" >&2
  echo "  npm run dev" >&2
  echo "" >&2
  echo "Optional (helps macOS find Java):" >&2
  echo "  sudo ln -sfn \$(brew --prefix openjdk@21)/libexec/openjdk.jdk /Library/Java/JavaVirtualMachines/openjdk-21.jdk" >&2
  exit 1
fi
