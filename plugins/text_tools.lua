local b = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'

function base64_encode(data)
    return ((data:gsub('.', function(x) 
        local r, b='', x:byte()
        for i=8,1,-1 do r=r..(b%2^i-b%2^(i-1)>0 and '1' or '0') end
        return r;
    end)..'0000'):gsub('%d%d%d?%d?%d?%d?', function(x)
        if (#x < 6) then return '' end
        local c=0
        for i=1,6 do c=c+(x:sub(i,i)=='1' and 2^(6-i) or 0) end
        return b:sub(c+1,c+1)
    end)..({ '', '==', '=' })[#data%3+1])
end

function process_text(action, text)
    if action == "upper" then
        return text:upper()
    elseif action == "lower" then
        return text:lower()
    elseif action == "reverse" then
        return text:reverse()
    elseif action == "length" then
        return tostring(#text)
    elseif action == "base64" then
        return base64_encode(text)
    else
        return text
    end
end
