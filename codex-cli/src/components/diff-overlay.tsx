import { Box, Text, useInput } from "ink";
import React, { useState } from "react";

/**
 * Simple scrollable view for displaying a diff.
 * The component is intentionally lightweight and mirrors the UX of
 * HistoryOverlay: Up/Down or j/k to scroll, PgUp/PgDn for paging and Esc to
 * close. The caller is responsible for computing the diff text.
 */
export default function DiffOverlay({
  diffText,
  onExit,
}: {
  diffText: string;
  onExit: () => void;
}): JSX.Element {
  const lines = diffText.length > 0 ? diffText.split("\n") : ["(no changes)"];

  const [cursor, setCursor] = useState(0);

  // Determine how many rows we can display – similar to HistoryOverlay.
  const rows = process.stdout.rows || 24;
  const headerRows = 2;
  const footerRows = 1;
  const maxVisible = Math.max(4, rows - headerRows - footerRows);

  useInput((input, key) => {
    if (key.escape || input === "q") {
      onExit();
      return;
    }

    if (key.downArrow || input === "j") {
      setCursor((c) => Math.min(lines.length - 1, c + 1));
    } else if (key.upArrow || input === "k") {
      setCursor((c) => Math.max(0, c - 1));
    } else if (key.pageDown) {
      setCursor((c) => Math.min(lines.length - 1, c + maxVisible));
    } else if (key.pageUp) {
      setCursor((c) => Math.max(0, c - maxVisible));
    } else if (input === "g") {
      setCursor(0);
    } else if (input === "G") {
      setCursor(lines.length - 1);
    }
  });

  const firstVisible = Math.min(
    Math.max(0, cursor - Math.floor(maxVisible / 2)),
    Math.max(0, lines.length - maxVisible),
  );
  const visible = lines.slice(firstVisible, firstVisible + maxVisible);

  // Very small helper to colorize diff lines in a basic way.
  function renderLine(line: string, idx: number): JSX.Element {
    // Preserve leading indentation but apply color to the diff markers and content.
    const match = line.match(/^(\s*)(.*)$/);
    const indent = match ? match[1] : "";
    const content = match ? match[2] : line;
    let color: "green" | "red" | "cyan" | undefined;
    if (content.startsWith("+")) {
      color = "green";
    } else if (content.startsWith("-")) {
      color = "red";
    } else if (content.startsWith("@@") || content.startsWith("diff --git")) {
      color = "cyan";
    }
    // Render indent uncolored and apply color only to content if applicable.
    return (
      <Text key={idx} wrap="truncate-end">
        {indent}
        <Text color={color}>{content === "" ? " " : content}</Text>
      </Text>
    );
  }

  return (
    <Box
      flexDirection="column"
      borderStyle="round"
      borderColor="gray"
      width={Math.min(120, process.stdout.columns || 120)}
    >
      <Box paddingX={1}>
        <Text bold>Working tree diff ({lines.length} lines)</Text>
      </Box>

      <Box flexDirection="column" paddingX={1}>
        {visible.map((line, idx) => {
          return renderLine(line, firstVisible + idx);
        })}
      </Box>

      <Box paddingX={1}>
        <Text dimColor>esc Close ↑↓ Scroll PgUp/PgDn g/G First/Last</Text>
      </Box>
    </Box>
  );
}
