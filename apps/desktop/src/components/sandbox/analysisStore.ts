import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';

interface StoryResult {
    story: string;
    driftX: number;
    driftY: number;
    shearX: number;
    shearY: number;
}

interface AnalysisState {
    results: StoryResult[];
    selectedStory: string | null;
    filterExceedingLimit: boolean;
    driftLimit: number;
    setSelectedStory: (story: string | null) => void;
    toggleFilter: () => void;
    setDriftLimit: (limit: number) => void;
    addResult: (r: StoryResult) => void;
    reset: () => void;
}

const mockResults: StoryResult[] = [
    { story: 'B1', driftX: 0.00, driftY: 0.00, shearX: 1200, shearY: 1100 },
    { story: 'GF', driftX: 0.12, driftY: 0.10, shearX: 1050, shearY: 980  },
    { story: 'L1', driftX: 0.45, driftY: 0.38, shearX: 880,  shearY: 820  },
    { story: 'L2', driftX: 0.68, driftY: 0.59, shearX: 720,  shearY: 670  },
    { story: 'L3', driftX: 0.82, driftY: 0.74, shearX: 580,  shearY: 540  },
    { story: 'L4', driftX: 0.91, driftY: 0.88, shearX: 450,  shearY: 420  },
    { story: 'L5', driftX: 0.85, driftY: 0.83, shearX: 330,  shearY: 310  },
    { story: 'L6', driftX: 0.76, driftY: 0.70, shearX: 220,  shearY: 200  },
    { story: 'L7', driftX: 0.61, driftY: 0.55, shearX: 120,  shearY: 110  },
    { story: 'L8', driftX: 0.38, driftY: 0.32, shearX: 45,   shearY: 40   },
];

export const useAnalysisStore = create<AnalysisState>()(
    immer((set) => ({
        results: mockResults,
        selectedStory: null,
        filterExceedingLimit: false,
        driftLimit: 0.5,

        setSelectedStory: (story) => set((state) => {
            state.selectedStory = story;
        }),

        toggleFilter: () => set((state) => {
            state.filterExceedingLimit = !state.filterExceedingLimit;
        }),

        setDriftLimit: (limit) => set((state) => {
            state.driftLimit = limit;
        }),

        addResult: (r) => set((state) => {
            state.results.push(r);
        }),

        reset: () => set((state) => {
            state.results = mockResults;
            state.selectedStory = null;
            state.filterExceedingLimit = false;
            state.driftLimit = 0.5;
        }),
    }))
);
