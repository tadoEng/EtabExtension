import { useRef } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Search, Layers, ArrowUpDown } from 'lucide-react';
import {
    flexRender,
    getCoreRowModel,
    useReactTable,
    getSortedRowModel,
} from '@tanstack/react-table';
import { useVirtualizer } from '@tanstack/react-virtual';

const MOCK_INVENTORY = Array.from({ length: 1000 }, (_, i) => ({
    id: `E-${1000 + i}`,
    name: `Structural Component ${i}`,
    status: i % 3 === 0 ? "Installed" : i % 3 === 1 ? "Pending" : "Ordered",
    load: `${Math.floor(Math.random() * 5000)}kN`,
    material: i % 2 === 0 ? "S355 Steel" : "C40/50 Concrete",
}));

const columns = [
    { accessorKey: 'id', header: 'ID', size: 100 },
    { accessorKey: 'name', header: 'Component Name', size: 200 },
    { accessorKey: 'status', header: 'Status', size: 120 },
    { accessorKey: 'load', header: 'Load Capacity', size: 120 },
    { accessorKey: 'material', header: 'Material', size: 150 },
];

export function BimDataTable() {
    const tableContainerRef = useRef<HTMLDivElement>(null);

    const table = useReactTable({
        data: MOCK_INVENTORY,
        columns,
        getCoreRowModel: getCoreRowModel(),
        getSortedRowModel: getSortedRowModel(),
    });

    const rows = table.getRowModel().rows;

    const rowVirtualizer = useVirtualizer({
        count: rows.length,
        getScrollElement: () => tableContainerRef.current,
        estimateSize: () => 40,
    });

    return (
        <Card className="h-full flex flex-col border-border/50 overflow-hidden">
            <CardHeader className="pb-3">
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        <Layers className="w-5 h-5 text-primary" />
                        <div>
                            <CardTitle className="text-sm">BIM Data Explorer</CardTitle>
                            <CardDescription>{MOCK_INVENTORY.length} structural components</CardDescription>
                        </div>
                    </div>
                    <div className="relative w-64">
                        <Search className="w-4 h-4 absolute left-3 top-2.5 text-muted-foreground" />
                        <input
                            className="w-full bg-background border border-border rounded-md pl-9 pr-4 py-1.5 text-xs focus:ring-1 focus:ring-primary outline-none"
                            placeholder="Filter components..."
                        />
                    </div>
                </div>
            </CardHeader>
            <CardContent className="flex-1 p-0 overflow-hidden border-t border-border/50">
                <div ref={tableContainerRef} className="h-full overflow-auto">
                    <table className="w-full text-left text-sm border-collapse">
                        <thead className="sticky top-0 bg-secondary/80 backdrop-blur z-20">
                        {table.getHeaderGroups().map(hg => (
                            <tr key={hg.id} className="border-b border-border/50">
                                {hg.headers.map(header => (
                                    <th
                                        key={header.id}
                                        onClick={header.column.getToggleSortingHandler()}
                                        className="px-4 py-3 font-semibold cursor-pointer hover:bg-accent transition text-left"
                                        style={{ width: header.getSize() }}
                                    >
                                        <div className="flex items-center gap-2">
                                            {flexRender(header.column.columnDef.header, header.getContext())}
                                            <ArrowUpDown className="w-3 h-3 opacity-50" />
                                        </div>
                                    </th>
                                ))}
                            </tr>
                        ))}
                        </thead>
                        <tbody style={{ height: `${rowVirtualizer.getTotalSize()}px`, position: 'relative' }}>
                        {rowVirtualizer.getVirtualItems().map(virtualRow => {
                            const row = rows[virtualRow.index];
                            return (
                                <tr
                                    key={row.id}
                                    style={{
                                        position: 'absolute',
                                        top: 0,
                                        left: 0,
                                        width: '100%',
                                        transform: `translateY(${virtualRow.start}px)`,
                                        height: `${virtualRow.size}px`
                                    }}
                                    className="border-b border-border/20 hover:bg-accent/30 transition-colors"
                                >
                                    {row.getVisibleCells().map(cell => (
                                        <td
                                            key={cell.id}
                                            className="px-4 py-2 text-xs"
                                            style={{ width: cell.column.getSize() }}
                                        >
                                            {flexRender(cell.column.columnDef.cell, cell.getContext())}
                                        </td>
                                    ))}
                                </tr>
                            );
                        })}
                        </tbody>
                    </table>
                </div>
            </CardContent>
        </Card>
    );
}