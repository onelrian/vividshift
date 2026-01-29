export interface Person {
    id: number;
    name: string;
    group_type: string;
    active: boolean;
}

export interface Assignment {
    id: number;
    person_id: number;
    task_name: string;
    assigned_at: string;
}

export interface DashboardData {
    people: Person[];
    recent_assignments: Assignment[];
    next_shuffle_in_days: number;
}

export interface Setting {
    key: string;
    value: string;
}
