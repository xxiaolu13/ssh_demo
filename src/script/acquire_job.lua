-- acquire_job.lua

-- KEYS[1]: pending_queue (等待执行的任务 ZSet)
-- KEYS[2]: processing_queue (正在执行的任务 ZSet，用于容错)
-- ARGV[1]: current_ts (当前时间戳，毫秒)
-- ARGV[2]: timeout_ms (任务超时时间，比如 30000ms)

local pending_key = KEYS[1]
local processing_key = KEYS[2]
local current_ts = tonumber(ARGV[1])
local timeout_ms = tonumber(ARGV[2])

-- 1. 查询 Pending 队列中，分数小于等于当前时间的第一个任务
-- ZRANGEBYSCORE key min max LIMIT offset count
local jobs = redis.call('ZRANGEBYSCORE', pending_key, '-inf', current_ts, 'LIMIT', 0, 1)

if #jobs > 0 then
    local job_id = jobs[1]

    -- 2. 计算超时死线 (Deadline)
    local deadline = current_ts + timeout_ms

    -- 3. 原子移动：先从 Pending 删掉，再加到 Processing
    redis.call('ZREM', pending_key, job_id)
    redis.call('ZADD', processing_key, deadline, job_id)

    -- 4. 返回抢到的 Job ID
    return job_id
else
    -- 没任务，返回空
    return nil
end
