import ReactECharts from 'echarts-for-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { BarChart3 } from 'lucide-react';

const chartOption = {
    title: { text: 'Performance Metrics', textStyle: { color: '#ffffff' } },
    tooltip: {
        trigger: 'axis',
        backgroundColor: 'rgba(0, 0, 0, 0.8)',
        borderColor: '#333',
        textStyle: { color: '#fff' }
    },
    legend: {
        data: ['Build Time', 'Load Time', 'Parse Time'],
        textStyle: { color: '#666' },
        bottom: 0
    },
    grid: { left: '3%', right: '4%', bottom: '15%', containLabel: true },
    xAxis: {
        type: 'category',
        data: ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'],
        axisLine: { lineStyle: { color: '#333' } },
        axisLabel: { color: '#666' }
    },
    yAxis: {
        type: 'value',
        axisLine: { lineStyle: { color: '#333' } },
        axisLabel: { color: '#666' },
        splitLine: { lineStyle: { color: '#222' } }
    },
    series: [
        { name: 'Build Time', data: [120, 132, 101, 134, 90, 230, 210], type: 'bar', itemStyle: { color: '#3b82f6' } },
        { name: 'Load Time', data: [220, 182, 191, 234, 290, 330, 310], type: 'bar', itemStyle: { color: '#10b981' } },
        { name: 'Parse Time', data: [150, 232, 201, 154, 190, 330, 410], type: 'bar', itemStyle: { color: '#f59e0b' } }
    ]
};

export function PerformanceChart() {
    return (
        <Card className="h-full border-border/50 flex flex-col">
            <CardHeader>
                <div className="flex items-center gap-2">
                    <BarChart3 className="w-5 w-5 text-primary" />
                    <div>
                        <CardTitle className="text-sm">Performance Analytics</CardTitle>
                        <CardDescription>Build, load, and parse time metrics</CardDescription>
                    </div>
                </div>
            </CardHeader>
            <CardContent className="flex-1 p-0 border-t border-border/50">
                <ReactECharts
                    option={chartOption}
                    style={{ width: '100%', height: '100%' }}
                    opts={{ renderer: 'canvas' }}
                />
            </CardContent>
        </Card>
    );
}