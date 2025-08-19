export function formatModuleCode(code: string): string {
  // Match one or more letters followed by one or more digits
  const match = code.match(/^([A-Za-z]+)(\d+)$/);
  if (!match) return code; // Return unchanged if it doesn't match the expected pattern

  const [, letters, numbers] = match;
  return `${letters} ${numbers}`;
}