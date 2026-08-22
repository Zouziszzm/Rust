"use client";

import { FormEvent, useCallback, useEffect, useState } from "react";
import {
  Todo,
  createTodo,
  deleteTodo,
  listTodos,
  updateTodo,
} from "@/lib/api";
import styles from "./TodoApp.module.css";

export default function TodoApp() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [title, setTitle] = useState("");
  const [description, setDescription] = useState("");
  const [loading, setLoading] = useState(true);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setError(null);
    const data = await listTodos();
    setTodos(data.items);
  }, []);

  useEffect(() => {
    refresh()
      .catch((err: Error) => setError(err.message))
      .finally(() => setLoading(false));
  }, [refresh]);

  async function onCreate(event: FormEvent) {
    event.preventDefault();
    if (!title.trim() || busy) return;

    setBusy(true);
    setError(null);
    try {
      await createTodo({
        title: title.trim(),
        description: description.trim() || undefined,
      });
      setTitle("");
      setDescription("");
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to create todo");
    } finally {
      setBusy(false);
    }
  }

  async function onToggle(todo: Todo) {
    setBusy(true);
    setError(null);
    try {
      await updateTodo(todo.id, { completed: !todo.completed });
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to update todo");
    } finally {
      setBusy(false);
    }
  }

  async function onDelete(id: string) {
    setBusy(true);
    setError(null);
    try {
      await deleteTodo(id);
      await refresh();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to delete todo");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className={styles.shell}>
      <header className={styles.header}>
        <p className={styles.brand}>odot</p>
        <h1 className={styles.headline}>Todos</h1>
      </header>

      <form className={styles.form} onSubmit={onCreate}>
        <label className={styles.label}>
          Title
          <input
            className={styles.input}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="What needs doing?"
            maxLength={255}
            required
          />
        </label>
        <label className={styles.label}>
          Description
          <textarea
            className={styles.textarea}
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Optional notes"
            rows={2}
            maxLength={4096}
          />
        </label>
        <button className={styles.button} type="submit" disabled={busy}>
          Add todo
        </button>
      </form>

      {error && <p className={styles.error}>{error}</p>}
      {loading ? (
        <p className={styles.muted}>Loading…</p>
      ) : todos.length === 0 ? (
        <p className={styles.muted}>No todos yet. Add one above.</p>
      ) : (
        <ul className={styles.list}>
          {todos.map((todo) => (
            <li
              key={todo.id}
              className={`${styles.item} ${todo.completed ? styles.done : ""}`}
            >
              <button
                type="button"
                className={styles.check}
                onClick={() => onToggle(todo)}
                disabled={busy}
                aria-label={todo.completed ? "Mark incomplete" : "Mark complete"}
              >
                {todo.completed ? "✓" : ""}
              </button>
              <div className={styles.body}>
                <p className={styles.title}>{todo.title}</p>
                {todo.description && (
                  <p className={styles.description}>{todo.description}</p>
                )}
              </div>
              <button
                type="button"
                className={styles.delete}
                onClick={() => onDelete(todo.id)}
                disabled={busy}
              >
                Delete
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
