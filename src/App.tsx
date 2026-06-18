import { useEffect, useMemo, useRef, useState } from "react";
import { commands, events } from "./bindings";
import type { CoreEvent, Message, Room, Space, User } from "./bindings";

async function call<T>(
  p: Promise<{ status: "ok"; data: T } | { status: "error"; error: unknown }>,
): Promise<T | null> {
  const r = await p;
  return r.status === "ok" ? r.data : null;
}

function shortName(userId: string): string {
  // "@alice:vauxl.local" -> "alice"
  return userId.replace(/^@/, "").split(":")[0];
}

function bodyOf(m: Message): string {
  const c = m.content;
  switch (c.type) {
    case "Text":
      return c.body;
    case "Image":
      return c.alt ?? "[image]";
    case "File":
      return `[file] ${c.filename}`;
    case "Audio":
      return "[voice message]";
    case "Redacted":
      return "(deleted)";
    default:
      return "[unsupported]";
  }
}

function roomGlyph(kind: Room["kind"]): string {
  switch (kind) {
    case "Voice":
      return "\u{1F50A}"; // speaker
    case "Announcement":
      return "\u{1F4E3}"; // megaphone
    case "DirectMessage":
      return "@";
    default:
      return "#";
  }
}

function encryptionBadge(enc: Room["encryption"]): string {
  switch (enc) {
    case "Encrypted":
      return "\u{1F512}"; // lock
    case "EncryptedUnverified":
      return "\u{26A0}"; // warning
    default:
      return "";
  }
}

export default function App() {
  const [me, setMe] = useState<User | null>(null);
  const [spaces, setSpaces] = useState<Space[]>([]);
  const [rooms, setRooms] = useState<Room[]>([]);
  const [activeSpace, setActiveSpace] = useState<string | null>(null);
  const [activeRoom, setActiveRoom] = useState<string | null>(null);
  const [messages, setMessages] = useState<Message[]>([]);
  const [typing, setTyping] = useState<string[]>([]);
  const [draft, setDraft] = useState("");

  const activeRoomRef = useRef<string | null>(null);
  const scrollRef = useRef<HTMLDivElement | null>(null);

  // Initial load.
  useEffect(() => {
    (async () => {
      const session = await call(commands.restoreSession());
      if (session) setMe(session.user);

      const sp = (await call(commands.listSpaces())) ?? [];
      const rm = (await call(commands.listRooms())) ?? [];
      setSpaces(sp);
      setRooms(rm);

      const firstSpace = sp[0]?.id ?? null;
      setActiveSpace(firstSpace);
      const firstRoom = rm.find((r) => r.space === firstSpace && r.kind === "Text");
      if (firstRoom) void openRoom(firstRoom.id);
    })();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Reactive core events.
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void events.coreEventMsg.listen((e) => handleEvent(e.payload.event)).then((u) => {
      unlisten = u;
    });
    return () => unlisten?.();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Keep pinned to the newest message.
  useEffect(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messages]);

  function handleEvent(ev: CoreEvent) {
    if (ev.type === "Timeline" && ev.room === activeRoomRef.current) {
      const ch = ev.change;
      if (ch.op === "Added") setMessages((prev) => [...prev, ch.message]);
      else if (ch.op === "Updated")
        setMessages((prev) => prev.map((m) => (m.id === ch.message.id ? ch.message : m)));
      else if (ch.op === "Removed") setMessages((prev) => prev.filter((m) => m.id !== ch.id));
    } else if (ev.type === "Typing" && ev.room === activeRoomRef.current) {
      setTyping(ev.users);
    }
  }

  async function openRoom(roomId: string) {
    setActiveRoom(roomId);
    activeRoomRef.current = roomId;
    setTyping([]);
    const chunk = await call(commands.loadTimeline(roomId, 50));
    setMessages(chunk?.messages ?? []);
  }

  async function send() {
    const room = activeRoom;
    const body = draft.trim();
    if (!room || !body) return;
    setDraft("");
    // The mock echoes the message back as a Timeline event, so we do not append here.
    await commands.sendMessage(room, { type: "Text", body, formatted: null, reply_to: null });
  }

  const roomsInSpace = useMemo(
    () => rooms.filter((r) => r.space === activeSpace),
    [rooms, activeSpace],
  );
  const dms = useMemo(() => rooms.filter((r) => r.space === null), [rooms]);
  const current = rooms.find((r) => r.id === activeRoom) ?? null;

  return (
    <div className="flex h-full bg-zinc-900 text-zinc-100">
      {/* Spaces rail */}
      <nav className="flex w-[72px] flex-col items-center gap-2 bg-zinc-950 py-3">
        {spaces.map((s) => {
          const active = s.id === activeSpace;
          return (
            <button
              key={s.id}
              title={s.name}
              onClick={() => {
                setActiveSpace(s.id);
                const first = rooms.find((r) => r.space === s.id && r.kind === "Text");
                if (first) void openRoom(first.id);
              }}
              className={`flex h-12 w-12 items-center justify-center rounded-2xl text-lg font-semibold transition-all ${
                active
                  ? "rounded-xl bg-indigo-600 text-white"
                  : "bg-zinc-800 text-zinc-300 hover:rounded-xl hover:bg-indigo-600 hover:text-white"
              }`}
            >
              {s.name.slice(0, 2)}
            </button>
          );
        })}
      </nav>

      {/* Channel sidebar */}
      <aside className="flex w-60 flex-col bg-zinc-800">
        <header className="flex h-12 items-center border-b border-black/20 px-4 font-semibold shadow-sm">
          {spaces.find((s) => s.id === activeSpace)?.name ?? "Vauxl"}
        </header>
        <div className="flex-1 overflow-y-auto px-2 py-3">
          <ChannelGroup label="Channels" rooms={roomsInSpace} activeRoom={activeRoom} onOpen={openRoom} />
          {dms.length > 0 && (
            <div className="mt-4">
              <ChannelGroup label="Direct Messages" rooms={dms} activeRoom={activeRoom} onOpen={openRoom} />
            </div>
          )}
        </div>
        {me && (
          <footer className="flex h-14 items-center gap-2 bg-zinc-950/60 px-3">
            <div className="flex h-8 w-8 items-center justify-center rounded-full bg-indigo-600 text-sm font-bold">
              {me.display_name.slice(0, 1).toUpperCase()}
            </div>
            <div className="leading-tight">
              <div className="text-sm font-medium">{me.display_name}</div>
              <div className="text-xs text-zinc-400">{me.presence}</div>
            </div>
          </footer>
        )}
      </aside>

      {/* Main */}
      <main className="flex flex-1 flex-col bg-zinc-700/40">
        <header className="flex h-12 items-center gap-2 border-b border-black/20 px-4 shadow-sm">
          <span className="text-zinc-400">{current ? roomGlyph(current.kind) : ""}</span>
          <span className="font-semibold">{current?.name ?? "Select a channel"}</span>
          {current?.encryption && (
            <span title={current.encryption} className="text-sm">
              {encryptionBadge(current.encryption)}
            </span>
          )}
          {current?.topic && (
            <span className="ml-2 border-l border-zinc-600 pl-3 text-sm text-zinc-400">
              {current.topic}
            </span>
          )}
          <span className="ml-auto text-sm text-zinc-400">
            {current ? `${current.member_count} members` : ""}
          </span>
        </header>

        <div ref={scrollRef} className="flex-1 space-y-3 overflow-y-auto px-4 py-4">
          {messages.length === 0 && (
            <div className="mt-10 text-center text-sm text-zinc-500">
              No messages yet. Say something.
            </div>
          )}
          {messages.map((m) => {
            const mine = me?.id === m.sender;
            return (
              <div key={m.id} className="flex gap-3">
                <div
                  className={`mt-0.5 flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-sm font-bold ${
                    mine ? "bg-indigo-600" : "bg-zinc-600"
                  }`}
                >
                  {shortName(m.sender).slice(0, 1).toUpperCase()}
                </div>
                <div className="min-w-0">
                  <div className="flex items-baseline gap-2">
                    <span className="font-medium">{shortName(m.sender)}</span>
                    <span className="text-xs text-zinc-500">
                      {new Date(m.timestamp ?? 0).toLocaleTimeString([], {
                        hour: "2-digit",
                        minute: "2-digit",
                      })}
                    </span>
                    {m.send_state.type === "Sending" && (
                      <span className="text-xs text-zinc-500">sending…</span>
                    )}
                  </div>
                  <div className="whitespace-pre-wrap break-words text-zinc-200">{bodyOf(m)}</div>
                </div>
              </div>
            );
          })}
        </div>

        <div className="h-5 px-4 text-xs text-zinc-400">
          {typing.length > 0 && `${typing.map(shortName).join(", ")} is typing…`}
        </div>

        <form
          className="px-4 pb-4"
          onSubmit={(e) => {
            e.preventDefault();
            void send();
          }}
        >
          <input
            value={draft}
            onChange={(e) => setDraft(e.target.value)}
            disabled={!current || current.kind === "Voice"}
            placeholder={
              current ? `Message ${roomGlyph(current.kind)}${current.name}` : "Select a channel"
            }
            className="w-full rounded-lg bg-zinc-600/60 px-4 py-3 text-sm outline-none placeholder:text-zinc-400 focus:ring-2 focus:ring-indigo-500 disabled:opacity-50"
          />
        </form>
      </main>
    </div>
  );
}

function ChannelGroup(props: {
  label: string;
  rooms: Room[];
  activeRoom: string | null;
  onOpen: (id: string) => void;
}) {
  return (
    <div>
      <div className="px-2 pb-1 text-xs font-semibold uppercase tracking-wide text-zinc-400">
        {props.label}
      </div>
      <ul className="space-y-0.5">
        {props.rooms.map((r) => {
          const active = r.id === props.activeRoom;
          const unread = r.unread.unread > 0;
          return (
            <li key={r.id}>
              <button
                onClick={() => props.onOpen(r.id)}
                className={`flex w-full items-center gap-2 rounded px-2 py-1.5 text-sm transition-colors ${
                  active
                    ? "bg-zinc-600/70 text-white"
                    : "text-zinc-400 hover:bg-zinc-600/40 hover:text-zinc-200"
                }`}
              >
                <span className="text-zinc-500">{roomGlyph(r.kind)}</span>
                <span className={`truncate ${unread && !active ? "font-semibold text-zinc-100" : ""}`}>
                  {r.name}
                </span>
                {unread && (
                  <span className="ml-auto rounded-full bg-red-500 px-1.5 text-xs font-bold text-white">
                    {r.unread.unread}
                  </span>
                )}
              </button>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
