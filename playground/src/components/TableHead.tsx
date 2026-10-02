/** The column names of a table of files. */
export function TableHead({ columns }: { columns: string[] }) {
  return (
    <thead className="text-muted-foreground">
      <tr>{columns.map((column) => <th key={column} className="p-1">{column}</th>)}</tr>
    </thead>
  )
}
