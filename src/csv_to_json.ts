function remove_quotes(str: string): string {
  if (!str) {
    return '';
  }

  const size = str.length;

  if (str[0] === '"' && str[size - 1] === '"') {
    return str.substring(1, size - 1);
  }

  return str;
}

export type Result = Array<string[]> | Array<{[key: string]: string}>;

export async function read_csv_file(filename: string, has_headers: boolean): Promise<Result> {
  const csv_content = await Deno.readTextFile(filename);

  const lines: string[][] = [];
  let current_line: string[] = [];

  let start_idx: number = 0;
  let reading_quote: boolean = false;

  const size: number = csv_content.length;

  for (let i = 0; i < size; i += 1) {
    switch (csv_content[i]) {
      case '\n':
        if (!reading_quote) {
          const value = csv_content.substring(start_idx, i);
          current_line.push(remove_quotes(value));
          start_idx = i + 1;

          lines.push(current_line);
          current_line = [];
        }
        break;

      case '"':
        if (csv_content[i + 1] !== '"') {
          reading_quote = !reading_quote;
        } else {
          i += 1;
        }
        break;

      case ',':
        if (!reading_quote) {
          const value = csv_content.substring(start_idx, i);
          current_line.push(remove_quotes(value));
          start_idx = i + 1;
        }
        break;
    }
  }

  if (start_idx < size) {
    const value = csv_content.substring(start_idx, size);
    current_line.push(remove_quotes(value));

    lines.push(current_line);
    current_line = [];
  }

  if (has_headers && lines.length > 1) {
    const result: Array<{[key: string]: string}> = [];

    const headers = lines[0];

    const lines_size = lines.length;

    for (let i = 1; i < lines_size; i += 1) {
      const line = lines[i];

      const line_size = Math.max(line.length, headers.length);

      const data: any = {};

      for (let j = 0; j < line_size; j += 1) {
        const h = headers[j] ? headers[j].toLowerCase() : `_${j}`;

        data[h] = line[j] || '';
      }

      result.push(data);
    }

    return result;
  }

  return lines;
}
