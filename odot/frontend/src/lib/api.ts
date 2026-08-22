export type Todo = {
  id: string;
  title: string;
  description: string | null;
  completed: boolean;
  created_at: string;
  updated_at: string;
};

export type TodoListResponse = {
  items: Todo[];
  limit: number;
  offset: number;
  total: number;
};

export type ApiError = {
  error: string;
  code: string;
};

const API_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${API_URL}${path}`, {
    ...init,
    headers: {
      "Content-Type": "application/json",
      ...(init?.headers ?? {}),
    },
  });

  if (response.status === 204) {
    return undefined as T;
  }

  const body = await response.json().catch(() => null);

  if (!response.ok) {
    const message =
      body && typeof body === "object" && "error" in body
        ? String((body as ApiError).error)
        : `Request failed (${response.status})`;
    throw new Error(message);
  }

  return body as T;
}

export function listTodos(): Promise<TodoListResponse> {
  return request<TodoListResponse>("/todos?limit=100&offset=0");
}

export function createTodo(input: {
  title: string;
  description?: string;
}): Promise<Todo> {
  return request<Todo>("/todos", {
    method: "POST",
    body: JSON.stringify(input),
  });
}

export function updateTodo(
  id: string,
  input: { title?: string; description?: string; completed?: boolean },
): Promise<Todo> {
  return request<Todo>(`/todos/${id}`, {
    method: "PATCH",
    body: JSON.stringify(input),
  });
}

export function deleteTodo(id: string): Promise<void> {
  return request<void>(`/todos/${id}`, { method: "DELETE" });
}
