import { csvCell, csvFileName, toCsv } from './csv';

describe('CSV', () => {
  it('quotes the cells that need it, as RFC 4180 does', () => {
    expect(csvCell('plain')).toBe('plain');
    expect(csvCell('a, b')).toBe('"a, b"');
    expect(csvCell('say "hi"')).toBe('"say ""hi"""');
    expect(csvCell('two\nlines')).toBe('"two\nlines"');
    expect(csvCell(42)).toBe('42');
    expect(csvCell(false)).toBe('false');
    expect(csvCell(null)).toBe('');
    expect(csvCell(undefined)).toBe('');
  });

  it('neutralises a cell a spreadsheet would run as a formula', () => {
    expect(csvCell('=HYPERLINK("x")')).toBe(`"'=HYPERLINK(""x"")"`);
    expect(csvCell('+1')).toBe("'+1");
    expect(csvCell('-2')).toBe("'-2");
    expect(csvCell('@sum')).toBe("'@sum");
    expect(csvCell(-2)).toBe('-2');
  });

  it('writes a header line, then a line per row, each ended by CRLF', () => {
    const rows = [
      { email: 'lee@example.com', storage: 512 },
      { email: 'kim@example.com', storage: null },
    ];
    const csv = toCsv(rows, [
      { header: 'Email', value: (row) => row.email },
      { header: 'Storage', value: (row) => row.storage },
    ]);

    expect(csv).toBe('Email,Storage\r\nlee@example.com,512\r\nkim@example.com,\r\n');
  });

  it('names the file after the table, the environment and the day', () => {
    expect(csvFileName('users', 'production', new Date('2026-09-26T10:00:00Z'))).toBe(
      'users-production-2026-09-26.csv',
    );
  });
});
